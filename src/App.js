// App.js

import React, { useState, useEffect, useRef } from "react";
import "./App.css";
import { Network } from "vis-network";
import { DataSet } from "vis-data";

const { invoke } = window.__TAURI__.tauri;

const App = () => {
    const graphContainer = useRef(null);
    const [network, setNetwork] = useState(null);
    const [nodesData, setNodesData] = useState(new DataSet());
    const [edgesData, setEdgesData] = useState(new DataSet());

    useEffect(() => {
        const fetchGraph = async () => {
            try {
                const graphData = await invoke("get_graph");
                console.log("Graph data:", graphData);

                if (graphContainer.current) {
                    const nodes = new DataSet(
                        graphData.nodes.map((node) => ({
                            id: node.id.toString(), // Ensure id is a string
                            label: node.id.toString(),
                            available: node.availability,
                            color: node.availability
                                ? { background: "#5D8FDE", border: "#0E65ED" }
                                : { background: "#CCCCCC", border: "#666666" },
                        }))
                    );

                    // Create edges without duplicates
                    const edgeSet = new Set();

                    const edges = new DataSet(
                        graphData.edges
                            .filter(({ source, target }) => {
                                const edgeId = [source, target].sort().join("-");
                                if (edgeSet.has(edgeId)) {
                                    return false;
                                } else {
                                    edgeSet.add(edgeId);
                                    return true;
                                }
                            })
                            .map(({ source, target, cost }) => {
                                const edgeId = [source, target].sort().join("-");
                                return {
                                    id: edgeId,
                                    from: source,
                                    to: target,
                                    label: cost.toString(),
                                };
                            })
                    );

                    // Create the network
                    const networkInstance = new Network(
                        graphContainer.current,
                        { nodes, edges },
                        {
                            nodes: {
                                color: {
                                    background: "#5D8FDE",
                                    border: "#0E65ED"
                                },
                                shape: "circle",
                                font: {
                                    color: "#343434"
                                },
                            },
                            edges: {
                                color: "#848484",
                                width: 2,
                                font: {
                                    size: 12,
                                    color: "#000000",
                                    align: "top",
                                },
                                arrows: {
                                    to: { enabled: false }, // Ensure edges are undirected
                                },
                            },
                            physics: {
                                enabled: true,
                            },
                        }
                    );

                    setNetwork({ instance: networkInstance });
                    setNodesData(nodes);
                    setEdgesData(edges);
                }
            } catch (err) {
                console.error("Failed to fetch graph:", err);
            }
        };

        fetchGraph();
    }, []);

    const toggleNodeAvailability = async (nodeId) => {
        try {
            const currentNode = nodesData.get(nodeId);
            const newStatus = !currentNode.available;

            console.log("Node ID:", nodeId, "Type:", typeof nodeId);
            await invoke("set_node_availability", { id: nodeId, available: newStatus });

            nodesData.update({
                id: nodeId,
                available: newStatus,
                color: newStatus
                    ? { background: "#5D8FDE", border: "#0E65ED" }
                    : { background: "#CCCCCC", border: "#666666" },
            });

            resetGraph();
        } catch (err) {
            alert("Failed to toggle node availability: " + err);
        }
    };

    useEffect(() => {
        if (network) {
            network.instance.on("click", function (params) {
                if (params.nodes.length > 0) {
                    const nodeId = params.nodes[0].toString();
                    toggleNodeAvailability(nodeId);
                }
            });
        }
    }, [network, nodesData, edgesData]);

    const addEdge = async () => {
        const source = prompt("Enter source node ID:");
        const target = prompt("Enter target node ID:");
        const cost = parseInt(prompt("Enter edge cost:"), 10);

        if (source && target && !isNaN(cost)) {
            try {
                await invoke("add_edge", { source: source.toString(), target: target.toString(), cost });

                const edgeId = [source, target].sort().join("-");

                if (!edgesData.get(edgeId)) {
                    edgesData.add({
                        id: edgeId,
                        from: source.toString(),
                        to: target.toString(),
                        label: cost.toString(),
                    });
                } else {
                    alert("Edge already exists in the visualization.");
                }
            } catch (err) {
                alert("Failed to add edge: " + err);
            }
        } else {
            alert("Invalid input!");
        }
    };

    const addNode = async () => {
        const id = prompt("Enter node ID:");
        if (id) {
            try {
                await invoke("add_node", { id: id.toString() });

                nodesData.add({
                    id: id.toString(),
                    label: id.toString(),
                    available: true,
                    color: {
                        background: "#5D8FDE",
                        border: "#0E65ED",
                    },
                    title: `Click to toggle availability of ${id}`,
                });
            } catch (err) {
                alert("Failed to add node: " + err);
            }
        }
    };

    const removeNode = async () => {
        const id = prompt("Enter node ID to remove:");
        if (id) {
            try {
                await invoke("remove_node", { id: id.toString() });
                nodesData.remove({ id: id.toString() });
                // Remove associated edges from frontend
                const edgesToRemove = edgesData.get({
                    filter: (item) => item.from === id.toString() || item.to === id.toString(),
                }).map(edge => edge.id);
                edgesData.remove(edgesToRemove);
            } catch (err) {
                alert("Failed to remove node: " + err);
            }
        }
    };

    const removeEdge = async () => {
        const source = prompt("Enter source node ID:");
        const target = prompt("Enter target node ID:");

        if (source && target) {
            try {
                await invoke("remove_edge", { source: source.toString(), target: target.toString() });

                const edgeId = [source, target].sort().join("-");

                edgesData.remove(edgeId);
            } catch (err) {
                alert("Failed to remove edge: " + err);
            }
        } else {
            alert("Invalid input!");
        }
    };

    const findShortestPath = async () => {
        const start = prompt("Enter start node ID:");
        const target = prompt("Enter target node ID:");

        if (start && target) {
            try {
                const res = await invoke("route_packet", { start, target });

                if (res) {
                    const { path, cost } = res;
                    const pathEdges = [];
                    for (let i = 0; i < path.length - 1; i++) {
                        const edgeId = [path[i], path[i + 1]].sort().join("-");
                        pathEdges.push(edgeId);
                    }

                    console.log("Edges in best path:", pathEdges);

                    resetGraph();

                    pathEdges.forEach((edgeId) => {
                        const edge = edgesData.get(edgeId);
                        if (edge) {
                            edgesData.update({
                                id: edgeId,
                                color: {
                                    color: "red",
                                    highlight: "red",
                                    hover: "red",
                                },
                                width: 4,
                            });
                        } else {
                            console.warn(`Edge with ID ${edgeId} not found.`);
                        }
                    });
                } else {
                    alert("No path found!");
                }
            } catch (err) {
                console.error("Failed to find shortest path:", err);
                alert("Failed to find shortest path: " + err);
            }
        }
    };

    const routePacket = async () => {
        const start = prompt("Enter start node ID:");
        const target = prompt("Enter target node ID:");

        if (start && target) {
            try {
                const res = await invoke("route_packet", { start, target });

                if (res) {
                    const { path, cost } = res;
                    console.log("Path: ", path);
                    console.log("Cost: ", cost);

                    const pathEdges = [];
                    for (let i = 0; i < path.length - 1; i++) {
                        const edgeId = [path[i], path[i + 1]].sort().join("-");
                        pathEdges.push(edgeId);
                    }

                    // resetGraph();

                    pathEdges.forEach((edgeId) => {
                        const edge = edgesData.get(edgeId);
                        if (edge) {
                            edgesData.update({
                                id: edgeId,
                                color: {
                                    color: "green",
                                    highlight: "green",
                                    hover: "green",
                                },
                                width: 4,
                            });
                        } else {
                            console.warn(`Edge with ID ${edgeId} not found.`);
                        }
                    });
                } else {
                    alert("No available path found!");
                }
            } catch (err) {
                console.error("Failed to route packet:", err);
                alert("Failed to route packet: " + err);
            }
        }
    };

    const resetGraph = () => {
        if (edgesData) {
            edgesData.forEach((edge) => {
                edgesData.update({
                    id: edge.id,
                    color: {
                        color: "#848484",
                        highlight: "#848484",
                        hover: "#848484",
                    },
                    width: 2,
                });
            });
        }

        // nodesData.forEach((node) => {
        //     nodesData.update({
        //         id: node.id,
        //         color: {
        //             background: "#FFFFFF",
        //             border: "#000000",
        //         },
        //     });
        // });
    };

    return (
        <div className="app-container">
            <h1 className="app-title">Most Secure Path</h1>
            <div ref={graphContainer} className="graph-container"></div>
            <div className="toolbar">
                <button onClick={addNode}>Add Node</button>
                <button onClick={removeNode}>Remove Node</button>
                <button onClick={addEdge}>Add Edge</button>
                <button onClick={removeEdge}>Remove Edge</button>
                {/*<button onClick={findShortestPath}>Find Shortest Path</button>*/}
                <button onClick={routePacket}>Route Packet</button>
                <button onClick={resetGraph}>Reset Graph</button>
            </div>
        </div>
    );
};

export default App;
