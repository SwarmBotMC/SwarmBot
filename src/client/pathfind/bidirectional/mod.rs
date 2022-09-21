mod middleman;
mod path_constructor;

// pub async fn a_star_bi(
//     progressor: &impl Progressor<Unit=BlockLocation>,
//     heuristic: &impl Heuristic<BlockLocation>,
//     init_node: BlockLocation,
//     goal_check: &impl GoalCheck<Unit=BlockLocation>,
// ) -> Option<Path<BlockLocation>> {
//
//     if goal_check.is_goal(&init_node) {
//         return Some(Path {
//             inner: vec![init_node]
//         });
//     }
//
//     if init_node.dist2(goal_node) > (5 * 5) {
//         let middleman = Middleman::new();
//
//         let sender1 = middleman.node_sender.clone();
//         let sender2 = middleman.node_sender.clone();
//
//
//         let (forward, backward) = tokio::join!(
//             async {
//                 bi_path_helper(progressor, heuristic, init_node, goal_node, sender1, true).await
//             },
//             async {
//                 bi_path_helper(progressor, heuristic, goal_node, init_node, sender2, false).await
//             }
//         );
//
//         let split = middleman.get_split().await;
//
//         if let Some(edge) = forward.edge {
//             println!("edge {:?}", edge);
//             // build to the closest available
//             let path = PathConstructor::build_path_forward(&forward.track, edge);
//             return Some(Path {
//                 inner: path
//             });
//         }
//
//         if let Some(split) = split {
//             let ids = PathConstructor::build_path(&forward.track, &backward.track, split);
//
//             return Some(Path {
//                 inner: ids,
//             });
//         }
//     }
//
//     None
// }
//
// struct BidirectionalResult<T> {
//     did_fail: bool,
//     track: HashMap<BlockLocation, BlockLocation>,
//     edge: Option<T>,
// }
//
// async fn bi_path_helper(
//     progressor: &impl Progressor<Unit=BlockLocation>,
//     heuristic: &impl Heuristic<BlockLocation>,
//     init_node: BlockLocation,
//     goal_node: BlockLocation,
//     node_sender: tokio::sync::mpsc::Sender<middleman::Msg<BlockLocation>>,
//     forward: bool,
// ) -> BidirectionalResult<BlockLocation> {
//
//     // also is an explored
//     let mut g_scores = HashMap::new();
//     let mut queue = BinaryHeap::new();
//
//     let mut track = HashMap::new();
//
//     // init
//     g_scores.insert(init_node, 0f64);
//
//
//     let init_heap_node = HeapNode {
//         contents: init_node,
//         f_score: f64::MAX,
//     };
//
//
//     queue.push(init_heap_node);
//
//     while let Some(origin) = queue.pop() {
//         // let origin_node = origin.node;
//         if origin.contents == goal_node {
//             return BidirectionalResult {
//                 did_fail: false,
//                 track,
//                 edge: None,
//             };
//         }
//
//         let origin = origin.contents;
//         let origin_g_score = g_scores[&origin];
//
//         let progression = progressor.progressions(origin);
//
//         let progression = match progression {
//             Progression::Edge => {
//                 node_sender.send(middleman::Msg::Finished { forward }).await.unwrap();
//
//                 return BidirectionalResult {
//                     did_fail: true,
//                     track,
//                     edge: Some(origin),
//                 };
//             }
//             Progression::Movements(inner) => inner
//         };
//
//         for neighbor in progression {
//             let tentative_g_score = origin_g_score + neighbor.cost;
//             match g_scores.get_mut(&neighbor.value) {
//                 Some(prev_score) => if tentative_g_score < *prev_score {
//                     *prev_score = tentative_g_score;
//                 } else {
//                     continue;
//                 }
//                 None => {
//                     g_scores.insert(neighbor.value, tentative_g_score);
//                 }
//             };
//
//
//             let h_score = heuristic.heuristic(&neighbor.value);
//             let unique_add = track.insert(neighbor.value, origin).is_none();
//
//             if unique_add { // if this is the first time we added to the map
//
//                 let send_result = node_sender.send(middleman::Msg::Node(neighbor.value)).await;
//
//                 // this will be an error if the send channel has been closed (which means the middle man has found a collision), so we can stop
//                 if send_result.is_err() {
//                     return BidirectionalResult {
//                         did_fail: false,
//                         track,
//                         edge: None,
//                     };
//                 }
//             }
//
//
//             let f_score = tentative_g_score + h_score;
//
//             let heap_node = HeapNode {
//                 contents: neighbor.value,
//                 f_score,
//             };
//
//
//             queue.push(heap_node)
//         }
//     }
//     // we are finished
//     node_sender.send(middleman::Msg::Finished { forward }).await.unwrap();
//     BidirectionalResult {
//         did_fail: true,
//         track,
//         edge: None,
//     }
//     // track
// }
