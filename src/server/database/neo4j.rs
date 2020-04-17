use super::{get_env_string, DatabaseEndpoint, DatabasePool, QueryResult};
use crate::server::context::WarpgrapherRequestContext;
use crate::server::objects::{Node, Rel};
use crate::server::value::Value;
use crate::{Error, ErrorKind};
use juniper::FieldError;
use log::{debug, trace};
use r2d2_cypher::CypherConnectionManager;
use rusted_cypher::cypher::result::CypherResult;
use rusted_cypher::cypher::transaction::{Started, Transaction};
use rusted_cypher::Statement;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;

pub struct Neo4jEndpoint {
    db_url: String,
}

impl Neo4jEndpoint {
    pub fn from_env() -> Result<Neo4jEndpoint, Error> {
        Ok(Neo4jEndpoint {
            db_url: get_env_string("WG_NEO4J_URL")?,
        })
    }
}

impl DatabaseEndpoint for Neo4jEndpoint {
    fn get_pool(&self) -> Result<DatabasePool, Error> {
        let manager = CypherConnectionManager {
            url: self.db_url.to_owned(),
        };

        Ok(DatabasePool::Neo4j(
            r2d2::Pool::builder()
                .max_size(num_cpus::get().try_into().unwrap_or(8))
                .build(manager)
                .map_err(|e| Error::new(ErrorKind::CouldNotBuildNeo4jPool(e), None))?,
        ))
    }
}

pub struct Neo4jTransaction<'t> {
    transaction: Option<Transaction<'t, Started>>,
}

impl<'t> Neo4jTransaction<'t> {
    pub fn new(transaction: Transaction<'t, Started>) -> Neo4jTransaction {
        Neo4jTransaction {
            transaction: Some(transaction),
        }
    }
}

impl<'t> super::Transaction for Neo4jTransaction<'t> {
    type ImplQueryResult = Neo4jQueryResult;

    fn begin(&self) -> Result<(), FieldError> {
        debug!("transaction::begin called");
        Ok(())
    }

    fn commit(&mut self) -> Result<(), FieldError> {
        debug!("transaction::commit called");
        if let Some(t) = self.transaction.take() {
            t.commit().map(|_| Ok(()))?
        } else {
            Err(Error::new(ErrorKind::TransactionFinished, None).into())
        }
    }

    fn create_node(
        &mut self,
        label: &str,
        partition_key_opt: &Option<String>,
        props: HashMap<String, Value>,
    ) -> Result<Neo4jQueryResult, FieldError> {
        let query = String::from("CREATE (n:")
            + label
            + " { id: randomUUID() })\n"
            + "SET n += $props\n"
            + "RETURN n\n";
        let mut params = HashMap::new();
        params.insert("props".to_owned(), props.into());

        trace!(
            "Neo4jTransaction::create_node query statement query, params: {:#?}, {:#?}",
            query,
            params
        );
        let raw_results = self.exec(&query, partition_key_opt, Some(params));
        trace!(
            "Neo4jTransaction::create_node raw results: {:#?}",
            raw_results
        );
        raw_results
    }

    fn delete_nodes(&mut self, label: &str, force: bool, ids: Value, partition_key_opt: &Option<String>) -> Result<Neo4jQueryResult, FieldError> {
    let query = String::from("MATCH (n:")
        + label
        + ")\n"
        + "WHERE n.id IN $ids\n"
        + if force { "DETACH " } else { "" }
        + "DELETE n\n"
        + "RETURN count(*) as count\n";
    let mut params = HashMap::new();
    params.insert("ids".to_owned(), ids);

    trace!(
        "visit_node_delete_mutation_input query, params: {:#?}, {:#?}",
        query, params
    );
    let results = self.exec(&query, partition_key_opt, Some(params))?;
    trace!(
        "visit_node_delete_mutation_input Query results: {:#?}",
        results
    );

    Ok(results)
    }

    fn exec(
        &mut self,
        query: &str,
        _partition_key_opt: &Option<String>,
        params: Option<HashMap<String, Value>>,
    ) -> Result<Neo4jQueryResult, FieldError> {
        debug!(
            "transaction::exec called with query, params: {:#?}, {:#?}",
            query, params
        );
        if let Some(transaction) = self.transaction.as_mut() {
            let mut statement = Statement::new(String::from(query));
            if let Some(p) = params {
                for (k, v) in p.into_iter() {
                    statement.add_param::<String, &serde_json::Value>(k, &v.try_into()?)?;
                }
            }
            let result = transaction.exec(statement);
            debug!("transaction::exec result: {:#?}", result);
            Ok(Neo4jQueryResult::new(result?))
        } else {
            Err(Error::new(ErrorKind::TransactionFinished, None).into())
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn node_query_string(
        &mut self,
        query_string: &str,
        params: &mut HashMap<String, Value>,
        label: &str,
        var_suffix: &str,
        union_type: bool,
        return_node: bool,
        param_suffix: &str,
        props: HashMap<String, Value>,
    ) -> Result<String, FieldError> {
        trace!(
            "transaction::node_query_string called, union_type: {:#?}",
            union_type
        );

        let mut qs = query_string.to_string();

        if union_type {
            qs.push_str(&(String::from("MATCH (") + label + var_suffix + ")\n"));
        } else {
            qs.push_str(&(String::from("MATCH (") + label + var_suffix + ":" + label + ")\n"));
        }

        let mut wc = None;
        for k in props.keys() {
            match wc {
                None => {
                    wc = Some(
                        String::from("WHERE ")
                            + label
                            + var_suffix
                            + "."
                            + &k
                            + "=$"
                            + label
                            + param_suffix
                            + "."
                            + &k,
                    )
                }
                Some(wcs) => {
                    wc = Some(
                        wcs + " AND " + label + "." + &k + "=$" + label + param_suffix + "." + &k,
                    )
                }
            }
        }
        if let Some(wcs) = wc {
            qs.push_str(&(String::from(&wcs) + "\n"));
        }

        params.insert(String::from(label) + param_suffix, props.into());

        if return_node {
            qs.push_str(&(String::from("RETURN ") + label + var_suffix + "\n"));
        }

        Ok(qs)
    }

    fn rollback(&mut self) -> Result<(), FieldError> {
        debug!("transaction::rollback called");
        if let Some(t) = self.transaction.take() {
            Ok(t.rollback()?)
        } else {
            Err(Error::new(ErrorKind::TransactionFinished, None).into())
        }
    }
}

#[derive(Debug)]
pub struct Neo4jQueryResult {
    result: CypherResult,
}

impl Neo4jQueryResult {
    pub fn new(result: CypherResult) -> Neo4jQueryResult {
        Neo4jQueryResult { result }
    }
}

impl QueryResult for Neo4jQueryResult {
    fn get_nodes<GlobalCtx, ReqCtx>(
        self,
        name: &str,
    ) -> Result<Vec<Node<GlobalCtx, ReqCtx>>, FieldError>
    where
        GlobalCtx: Debug,
        ReqCtx: WarpgrapherRequestContext + Debug,
    {
        trace!("Neo4jQueryResult::get_nodes called");

        let mut v = Vec::new();
        for row in self.result.rows() {
            let m: HashMap<String, serde_json::Value> = row.get(name)?;
            let mut fields = HashMap::new();
            for (k, v) in m.into_iter() {
                fields.insert(k, v.try_into()?);
            }
            v.push(Node::new(name.to_owned(), fields));
        }
        trace!("Neo4jQueryResults::get_nodes results: {:#?}", v);
        Ok(v)
    }

    fn get_rels<GlobalCtx, ReqCtx>(
        self,
        src_name: &str,
        src_suffix: &str,
        rel_name: &str,
        dst_name: &str,
        dst_suffix: &str,
        props_type_name: Option<&str>,
    ) -> Result<Vec<Rel<GlobalCtx, ReqCtx>>, FieldError>
    where
        GlobalCtx: Debug,
        ReqCtx: WarpgrapherRequestContext + Debug,
    {
        trace!("Neo4jQueryResult::get_rels called, src_name, src_suffix, rel_name, dst_name, dst_suffix, props_type_name: {:#?}, {:#?}, {:#?}, {:#?}, {:#?}, {:#?}", src_name, src_suffix, rel_name, dst_name, dst_suffix, props_type_name);

        let mut v: Vec<Rel<GlobalCtx, ReqCtx>> = Vec::new();

        for row in self.result.rows() {
            if let serde_json::Value::Array(labels) =
                row.get(&(String::from(dst_name) + dst_suffix + "_label"))?
            {
                if let serde_json::Value::String(dst_type) = &labels[0] {
                    let src_map: HashMap<String, serde_json::Value> =
                        row.get::<HashMap<String, serde_json::Value>>(
                            &(String::from(src_name) + src_suffix),
                        )?;
                    let mut src_wg_map = HashMap::new();
                    for (k, v) in src_map.into_iter() {
                        src_wg_map.insert(k, v.try_into()?);
                    }

                    let dst_map: HashMap<String, serde_json::Value> =
                        row.get::<HashMap<String, serde_json::Value>>(
                            &(String::from(dst_name) + dst_suffix),
                        )?;
                    let mut dst_wg_map = HashMap::new();
                    for (k, v) in dst_map.into_iter() {
                        dst_wg_map.insert(k, v.try_into()?);
                    }

                    v.push(Rel::new(
                        row.get::<serde_json::Value>(
                            &(String::from(rel_name) + src_suffix + dst_suffix),
                        )?
                        .get("id")
                        .ok_or_else(|| {
                            Error::new(ErrorKind::MissingResultElement("id".to_string()), None)
                        })?
                        .clone()
                        .try_into()?,
                        match props_type_name {
                            Some(p_type_name) => {
                                let map: HashMap<String, serde_json::Value> =
                                    row.get::<HashMap<String, serde_json::Value>>(
                                        &(String::from(rel_name) + src_suffix + dst_suffix),
                                    )?;
                                let mut wg_map = HashMap::new();
                                for (k, v) in map.into_iter() {
                                    wg_map.insert(k, v.try_into()?);
                                }

                                Some(Node::new(p_type_name.to_string(), wg_map))
                            }
                            None => None,
                        },
                        Node::new(src_name.to_owned(), src_wg_map),
                        Node::new(dst_type.to_owned(), dst_wg_map),
                    ))
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidPropertyType(
                            String::from(dst_name) + dst_suffix + "_label",
                        ),
                        None,
                    )
                    .into());
                }
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidPropertyType(String::from(dst_name) + dst_suffix + "_label"),
                    None,
                )
                .into());
            };
        }
        trace!("Neo4jQueryResults::get_rels results: {:#?}", v);
        Ok(v)
    }

    fn get_ids(&self, name: &str) -> Result<Value, FieldError> {
        trace!("Neo4jQueryResult::get_ids called");

        let mut v = Vec::new();
        for row in self.result.rows() {
            let n: serde_json::Value = row.get(name)?;

            if let serde_json::Value::String(id) = n.get("id").ok_or_else(|| Error::new(ErrorKind::MissingProperty("id".to_owned(), Some("This is likely because a custom resolver created a node or rel without an id field.".to_owned())), None))? {
                v.push(Value::String(id.to_owned()));
            } else {
                return Err(Error::new(ErrorKind::InvalidPropertyType("id".to_owned()), None).into());
            }
        }

        trace!("get_ids result: {:#?}", v);
        Ok(Value::Array(v))
    }

    fn get_count(&self) -> Result<i32, FieldError> {
        trace!("Neo4jQueryResult::get_count called");

        let ret_row = self
            .result
            .rows()
            .next()
            .ok_or_else(|| Error::new(ErrorKind::MissingResultSet, None))?;
        let ret_val = ret_row
            .get("count")
            .map_err(|_| Error::new(ErrorKind::MissingResultElement("count".to_owned()), None))?;

        if let serde_json::Value::Number(n) = ret_val {
            if let Some(i_val) = n.as_i64() {
                Ok(i_val as i32)
            } else {
                Err(Error::new(ErrorKind::InvalidPropertyType("int".to_owned()), None).into())
            }
        } else {
            Err(Error::new(ErrorKind::InvalidPropertyType("int".to_owned()), None).into())
        }
    }

    fn len(&self) -> i32 {
        trace!("Neo4jQueryResult::len called");
        0
    }

    fn is_empty(&self) -> bool {
        trace!("Neo4jQueryResult::is_empty called");
        self.len() == 0
    }
}
