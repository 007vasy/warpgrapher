///! SNMT Relationship: board (KanbanBoard, ScrumBoard)
mod setup;

use serde_json::json;
#[cfg(feature = "graphson2")]
use setup::server::test_server_graphson2;
#[cfg(feature = "neo4j")]
use setup::server::test_server_neo4j;
#[cfg(any(feature = "graphson2", feature = "neo4j"))]
use setup::{clear_db, init};
#[cfg(feature = "graphson2")]
use setup::graphson2_test_client;
#[cfg(feature = "neo4j")]
use setup::neo4j_test_client;
use warpgrapher::client::Client;

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn create_node_with_rel_to_new_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    create_node_with_rel_to_new(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn create_node_with_rel_to_new_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    create_node_with_rel_to_new(client).await;

    assert!(server.shutdown().is_ok());
}

/// Passes if a node is created with an SNMT rel to a new node
#[allow(clippy::cognitive_complexity, dead_code)]
async fn create_node_with_rel_to_new(mut client: Client) {
    // create new Project with rel to new KanbanBoard
    let results0 = client
        .create_node(
            "Project",
            "__typename
            id
            board {
                __typename
                id
                dst {
                    ... on KanbanBoard {
                        __typename
                        id
                        name       
                    }
                }
            }",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN-V",
                "board": {
                    "dst": {
                        "KanbanBoard": {
                            "NEW": {
                                "name": "SPARTAN-V Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();

    assert!(results0.is_object());
    let p0_board = results0.get("board").unwrap();
    assert!(p0_board.is_object());
    assert_eq!(p0_board.get("__typename").unwrap(), "ProjectBoardRel");
    let p0_board_dst = p0_board.get("dst").unwrap();
    assert!(p0_board_dst.is_object());
    assert_eq!(p0_board_dst.get("__typename").unwrap(), "KanbanBoard");
    assert_eq!(p0_board_dst.get("name").unwrap(), "SPARTAN-V Board");

    // read Kanbanboard
    let results1 = client
        .read_node(
            "KanbanBoard",
            "__typename
            id
            name",
            Some("1234".to_string()),
            Some(&json!({
                "name": "SPARTAN-V Board"
            })),
        )
        .await
        .unwrap();

    assert!(results1.is_array());
    let b0 = &results1[0];
    assert!(b0.is_object());
    assert_eq!(b0.get("__typename").unwrap(), "KanbanBoard");
    assert_eq!(b0.get("name").unwrap(), "SPARTAN-V Board");

    // read Project
    let results2 = client
        .read_node(
            "Project",
            "__typename 
            id 
            name 
            board { 
                __typename 
                dst { 
                    ... on KanbanBoard {
                        __typename 
                        id
                        name
                    }
                } 
            }",
            Some("1234".to_string()),
            Some(&json!({
                "name": "SPARTAN-V"
            })),
        )
        .await
        .unwrap();

    assert!(results2.is_array());
    assert_eq!(results2.as_array().unwrap().len(), 1);
    let p1 = &results2[0];
    assert_eq!(p1.get("__typename").unwrap(), "Project");
    assert_eq!(p1.get("name").unwrap(), "SPARTAN-V");
    let p1_board = p1.get("board").unwrap();
    assert_eq!(p1_board.get("__typename").unwrap(), "ProjectBoardRel");
    let p1_board_dst = p1_board.get("dst").unwrap();
    assert_eq!(p1_board_dst.get("__typename").unwrap(), "KanbanBoard");
    assert_eq!(p1_board_dst.get("name").unwrap(), "SPARTAN-V Board");
    assert_eq!(p1_board_dst.get("id").unwrap(), b0.get("id").unwrap());
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn create_node_with_rel_to_existing_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    create_node_with_rel_to_existing(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn create_node_with_rel_to_existing_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    create_node_with_rel_to_existing(client).await;

    assert!(server.shutdown().is_ok());
}

/// Passes if a node is created with an SNMT rel to existing node
#[allow(clippy::cognitive_complexity, dead_code)]
async fn create_node_with_rel_to_existing(mut client: Client) {
    // create new ScrumBoard
    let results0 = client
        .create_node(
            "ScrumBoard",
            "__typename
            id
            name
            ",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN-VI Board"
            }),
        )
        .await
        .unwrap();
    assert!(results0.is_object());
    assert_eq!(results0.get("__typename").unwrap(), "ScrumBoard");

    // create new Project with rel to existing ScrumBoard
    let results1 = client
        .create_node(
            "Project",
            "__typename
            id
            board {
                __typename
                id
                dst {
                    ... on ScrumBoard {
                        __typename
                        id
                        name       
                    }
                }
            }",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN-VI",
                "board": {
                    "dst": {
                        "ScrumBoard": {
                            "EXISTING": {
                                "name": "SPARTAN-VI Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();

    assert!(results1.is_object());
    assert_eq!(results1.get("__typename").unwrap(), "Project");

    // read project
    let results2 = client
        .read_node(
            "Project",
            "__typename 
            id 
            name 
            board { 
                dst { 
                    ... on ScrumBoard {
                        __typename 
                        id
                        name
                    }
                } 
            }",
            Some("1234".to_string()),
            Some(&json!({
                "name": "SPARTAN-VI"
            })),
        )
        .await
        .unwrap();

    assert!(results2.is_array());
    assert_eq!(results2.as_array().unwrap().len(), 1);
    let p1 = &results2[0];
    assert_eq!(p1.get("__typename").unwrap(), "Project");
    let p1_board = p1.get("board").unwrap();
    let b1 = p1_board.get("dst").unwrap();
    assert_eq!(b1.get("__typename").unwrap(), "ScrumBoard");
    assert_eq!(b1.get("name").unwrap(), "SPARTAN-VI Board");
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn read_multiple_nodes_with_multiple_rels_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    read_multiple_nodes_with_multiple_rels(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn read_multiple_nodes_with_multiple_rels_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    read_multiple_nodes_with_multiple_rels(client).await;

    assert!(server.shutdown().is_ok());
}

/// Passes if multiple nodes with multiple rels are read and
/// the relationships associate correctly
#[allow(clippy::cognitive_complexity, dead_code)]
async fn read_multiple_nodes_with_multiple_rels(mut client: Client) {
    // create multiple nodes with multiple rels
    let results0 = client
        .create_node(
            "Project",
            "__typename
            id
            ",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN-10",
                "board": {
                    "dst": {
                        "ScrumBoard": {
                            "NEW": {
                                "name": "SPARTAN-10 Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();
    let results1 = client
        .create_node(
            "Project",
            "__typename
            id
            ",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN-11",
                "board": {
                    "dst": {
                        "KanbanBoard": {
                            "NEW": {
                                "name": "SPARTAN-11 Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();
    let results2 = client
        .create_node(
            "Project",
            "__typename
            id
            ",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN-12",
                "board": {
                    "dst": {
                        "KanbanBoard": {
                            "NEW": {
                                "name": "SPARTAN-12 Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();

    // read nodes
    let results3 = client
        .read_node(
            "Project",
            "__typename 
            id 
            name 
            board { 
                __typename 
                dst { 
                    __typename 
                    ... on ScrumBoard {
                        id
                        name
                    }
                    ... on KanbanBoard {
                        id
                        name
                    }
                } 
            }",
            Some("1234".to_string()),
            None,
        )
        .await
        .unwrap();

    assert!(results3.is_array());
    let projects = results3.as_array().unwrap();

    let p0 = projects
        .iter()
        .find(|&x| x.get("id").unwrap() == results0.get("id").unwrap())
        .unwrap();
    assert!(p0.is_object());
    let p0_board = p0.get("board").unwrap();
    assert_eq!(p0_board.get("__typename").unwrap(), "ProjectBoardRel");
    let b0 = p0_board.get("dst").unwrap();
    assert_eq!(b0.get("__typename").unwrap(), "ScrumBoard");
    assert_eq!(b0.get("name").unwrap(), "SPARTAN-10 Board");

    let p1 = projects
        .iter()
        .find(|&x| x.get("id").unwrap() == results1.get("id").unwrap())
        .unwrap();
    assert!(p1.is_object());
    let p1_board = p1.get("board").unwrap();
    assert_eq!(p1_board.get("__typename").unwrap(), "ProjectBoardRel");
    let b1 = p1_board.get("dst").unwrap();
    assert_eq!(b1.get("__typename").unwrap(), "KanbanBoard");
    assert_eq!(b1.get("name").unwrap(), "SPARTAN-11 Board");

    let p2 = projects
        .iter()
        .find(|&x| x.get("id").unwrap() == results2.get("id").unwrap())
        .unwrap();
    assert!(p2.is_object());
    let p2_board = p2.get("board").unwrap();
    assert_eq!(p2_board.get("__typename").unwrap(), "ProjectBoardRel");
    let b2 = p2_board.get("dst").unwrap();
    assert_eq!(b2.get("__typename").unwrap(), "KanbanBoard");
    assert_eq!(b2.get("name").unwrap(), "SPARTAN-12 Board");
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn read_node_with_matching_props_on_rel_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    read_node_with_matching_props_on_rel(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn read_node_with_matching_props_on_rel_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    read_node_with_matching_props_on_rel(client).await;

    assert!(server.shutdown().is_ok());
}

/// Passes if nodes matching props on a relationship are returned
#[allow(clippy::cognitive_complexity, dead_code)]
async fn read_node_with_matching_props_on_rel(mut client: Client) {
    // create nodes with rel with props
    let results0 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "ORION",
                "board": {
                    "props": {
                        "publicized": false
                    },
                    "dst": {
                        "ScrumBoard": {
                            "NEW": {
                                "name": "ORION Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();
    assert!(results0.is_object());
    let results1 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN",
                "board": {
                    "props": {
                        "publicized": true
                    },
                    "dst": {
                        "ScrumBoard": {
                            "NEW": {
                                "name": "SPARTAN Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();
    assert!(results1.is_object());

    // read projects matching props on rels
    let results3 = client
        .read_node(
            "Project",
            "__typename 
            id 
            name 
            board { 
                __typename 
                props {
                    publicized
                }
                dst { 
                    ... on ScrumBoard {
                        __typename 
                        id
                        name
                    }
                } 
            }",
            Some("1234".to_string()),
            Some(&json!({
                "board": {
                    "props": {
                        "publicized": true
                    }
                }
            })),
        )
        .await
        .unwrap();
    assert!(results3.is_array());
    let projects0 = results3.as_array().unwrap();
    assert_eq!(projects0.len(), 1);
    let p0 = &projects0[0];
    assert!(p0.is_object());
    assert_eq!(p0.get("__typename").unwrap(), "Project");
    assert_eq!(p0.get("name").unwrap(), "SPARTAN");
    let p0_board = p0.get("board").unwrap();
    assert_eq!(p0_board.get("__typename").unwrap(), "ProjectBoardRel");
    let b0 = p0_board.get("dst").unwrap();
    assert_eq!(b0.get("name").unwrap(), "SPARTAN Board");

    // read projects matching props on rels
    let results4 = client
        .read_node(
            "Project",
            "__typename 
            id 
            name 
            board { 
                __typename 
                dst { 
                    ... on ScrumBoard {
                        __typename 
                        id
                        name
                    }
                } 
            }",
            Some("1234".to_string()),
            Some(&json!({
                "board": {
                    "props": {
                        "publicized": false
                    }
                }
            })),
        )
        .await
        .unwrap();
    assert!(results4.is_array());
    let projects1 = results4.as_array().unwrap();
    assert_eq!(projects1.len(), 1);
    let p1 = &projects1[0];
    assert!(p1.is_object());
    assert_eq!(p1.get("__typename").unwrap(), "Project");
    assert_eq!(p1.get("name").unwrap(), "ORION");
    let p1_board = p1.get("board").unwrap();
    assert_eq!(p1_board.get("__typename").unwrap(), "ProjectBoardRel");
    let b1 = p1_board.get("dst").unwrap();
    assert_eq!(b1.get("name").unwrap(), "ORION Board");
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn read_node_with_matching_props_on_rel_dst_node_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    read_node_with_matching_props_on_rel_dst_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn read_node_with_matching_props_on_rel_dst_node_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    read_node_with_matching_props_on_rel_dst_node(client).await;

    assert!(server.shutdown().is_ok());
}

/// Passes if it returns nodes with relationship dst nodes
/// with matching props
#[allow(clippy::cognitive_complexity, dead_code)]
async fn read_node_with_matching_props_on_rel_dst_node(mut client: Client) {
    // create nodes with rel with props
    let _results0 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "ORION",
                "board": {
                    "props": {
                        "publicized": false
                    },
                    "dst": {
                        "ScrumBoard": {
                            "NEW": {
                                "name": "ORION Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();
    let _results1 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN",
                "board": {
                    "props": {
                        "publicized": false
                    },
                    "dst": {
                        "ScrumBoard": {
                            "NEW": {
                                "name": "SPARTAN Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();

    // read nodes matching rel dst node props
    let results2 = client
        .read_node(
            "Project",
            "__typename
             id
             name
             board {
                __typename 
                dst { 
                    ... on ScrumBoard {
                        __typename 
                        id
                        name
                    }
                    ... on KanbanBoard {
                        __typename 
                        id
                        name
                    }
                } 
             }
            ",
            Some("1234".to_string()),
            Some(&json!({
                "board": {
                    "dst": {
                        "ScrumBoard": {
                            "name": "SPARTAN Board"
                        }
                    }
                }
            })),
        )
        .await
        .unwrap();
    assert!(results2.is_array());
    let projects = results2.as_array().unwrap();
    assert_eq!(projects.len(), 1);
    let p0 = &projects[0];
    assert!(p0.is_object());
    assert_eq!(p0.get("__typename").unwrap(), "Project");
    assert_eq!(p0.get("name").unwrap(), "SPARTAN");
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn update_existing_node_with_rel_to_new_node_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    update_existing_node_with_rel_to_new_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn update_existing_node_with_rel_to_new_node_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    update_existing_node_with_rel_to_new_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[allow(clippy::cognitive_complexity, dead_code)]
async fn update_existing_node_with_rel_to_new_node(mut client: Client) {
    // create project node
    let _results0 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "ORION",
            }),
        )
        .await
        .unwrap();

    // update project node to create a rel to a new node
    let _results1 = client
        .update_node(
            "Project",
            "__typename
            id
            name
            board {
                dst {
                    ... on ScrumBoard {
                        __typename 
                        id
                        name
                    }
                }
            }
            ",
            Some("1234".to_string()),
            Some(&json!({
                "name": "ORION"
            })),
            &json!({
                "board": {
                    "ADD": {
                        "dst": {
                            "KanbanBoard": {
                                "NEW": {
                                    "name": "ORION Board"
                                }
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();

    // read nodes matching rel dst node props
    let results2 = client
        .read_node(
            "Project",
            "__typename
             id
             name
             board {
                 __typename
                 dst {
                    ... on ScrumBoard {
                        id
                        name
                    }
                    ... on KanbanBoard {
                        id
                        name
                    }
                }
             }
            ",
            Some("1234".to_string()),
            None,
        )
        .await
        .unwrap();
    assert!(results2.is_array());
    let projects = results2.as_array().unwrap();
    assert_eq!(projects.len(), 1);
    let p0 = &projects[0];
    assert!(p0.is_object());
    assert_eq!(p0.get("__typename").unwrap(), "Project");
    assert_eq!(p0.get("name").unwrap(), "ORION");
    let p0_board = p0.get("board").unwrap();
    assert_eq!(p0_board.get("__typename").unwrap(), "ProjectBoardRel");
    assert_eq!(
        p0_board.get("dst").unwrap().get("name").unwrap(),
        "ORION Board"
    );
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn update_existing_node_with_rel_to_existing_node_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    update_existing_node_with_rel_to_existing_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn update_existing_node_with_rel_to_existing_node_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    update_existing_node_with_rel_to_existing_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[allow(clippy::cognitive_complexity, dead_code)]
async fn update_existing_node_with_rel_to_existing_node(mut client: Client) {
    // create project node
    let _results0 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "ORION",
            }),
        )
        .await
        .unwrap();

    // create board node
    let _results1 = client
        .create_node(
            "ScrumBoard",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "ORION Board"
            }),
        )
        .await;

    // update project node to create a rel to a new node
    let _results2 = client
        .update_node(
            "Project",
            "__typename
            id
            name
            ",
            Some("1234".to_string()),
            Some(&json!({
                "name": "ORION"
            })),
            &json!({
                "board": {
                    "ADD": {
                        "dst": {
                            "ScrumBoard": {
                                "EXISTING": {
                                    "name": "ORION Board"
                                }
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();

    // read nodes matching rel dst node props
    let results2 = client
        .read_node(
            "Project",
            "__typename
             id
             name
             board {
                 __typename
                 dst {
                     ... on ScrumBoard {
                         id
                         name
                     }
                 }
             }
            ",
            Some("1234".to_string()),
            None,
        )
        .await
        .unwrap();
    assert!(results2.is_array());
    let projects = results2.as_array().unwrap();
    assert_eq!(projects.len(), 1);
    let p0 = &projects[0];
    assert!(p0.is_object());
    assert_eq!(p0.get("__typename").unwrap(), "Project");
    assert_eq!(p0.get("name").unwrap(), "ORION");
    let p0_board = p0.get("board").unwrap();
    assert_eq!(p0_board.get("__typename").unwrap(), "ProjectBoardRel");
    assert_eq!(
        p0_board.get("dst").unwrap().get("name").unwrap(),
        "ORION Board"
    );
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn delete_node_with_matching_props_on_rel_dst_node_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    delete_node_with_matching_props_on_rel_dst_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn delete_node_with_matching_props_on_rel_dst_node_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    delete_node_with_matching_props_on_rel_dst_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[allow(clippy::cognitive_complexity, dead_code)]
async fn delete_node_with_matching_props_on_rel_dst_node(mut client: Client) {
    // create project nodes
    let _results0 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "ORION",
            }),
        )
        .await
        .unwrap();
    let _results1 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "SPARTAN-II",
            }),
        )
        .await
        .unwrap();

    // delete node with matching props
    let _results2 = client
        .delete_node(
            "Project",
            Some("1234".to_string()),
            Some(&json!({"name": "ORION"})),
            None,
        )
        .await
        .unwrap();

    // read projects
    let results3 = client
        .read_node(
            "Project",
            "__typename
             id
             name
            ",
            Some("1234".to_string()),
            None,
        )
        .await
        .unwrap();
    assert!(results3.is_array());
    assert_eq!(results3.as_array().unwrap().len(), 1);
    assert_eq!(results3[0].get("name").unwrap(), "SPARTAN-II");
}

#[cfg(feature = "neo4j")]
#[tokio::test]
async fn delete_node_neo4j() {
    init();
    clear_db();

    let mut server = test_server_neo4j("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = neo4j_test_client();
    delete_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[cfg(feature = "graphson2")]
#[tokio::test]
async fn delete_node_graphson2() {
    init();
    clear_db();

    let mut server = test_server_graphson2("./tests/fixtures/minimal.yml");
    assert!(server.serve(false).is_ok());

    let client = graphson2_test_client();
    delete_node(client).await;

    assert!(server.shutdown().is_ok());
}

#[allow(clippy::cognitive_complexity, dead_code)]
async fn delete_node(mut client: Client) {
    // create project nodes
    let _results0 = client
        .create_node(
            "Project",
            "id",
            Some("1234".to_string()),
            &json!({
                "name": "ORION",
                "board": {
                    "dst": {
                        "ScrumBoard": {
                            "NEW": {
                                "name": "ORION Board"
                            }
                        }
                    }
                }
            }),
        )
        .await
        .unwrap();

    // delete node with matching props
    let _results1 = client
        .delete_node(
            "Project",
            Some("1234".to_string()),
            Some(&json!({"name": "ORION"})),
            Some(&json!({})),
        )
        .await
        .unwrap();

    // read projects
    let results2 = client
        .read_node(
            "Project",
            "__typename
             id
             name
            ",
            Some("1234".to_string()),
            None,
        )
        .await
        .unwrap();
    assert!(results2.is_array());
    assert_eq!(results2.as_array().unwrap().len(), 0);

    // verify dst node was not deleted
    let results3 = client
        .read_node(
            "ScrumBoard",
            "__typename
             id
             name
            ",
            Some("1234".to_string()),
            None,
        )
        .await
        .unwrap();
    assert!(results3.is_array());
    assert_eq!(results3.as_array().unwrap().len(), 1);
    assert_eq!(results3[0].get("name").unwrap(), "ORION Board");
}
