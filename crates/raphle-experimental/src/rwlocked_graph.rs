use std::{fs::File, io::BufReader, sync::RwLock};
use hashbrown::HashMap;
use roaring::bitmap::RoaringBitmap;
use csv::ReaderBuilder;
use tracing::info;

pub struct RwLockedNodeMap {
    outgoing_edges: RwLock<RoaringBitmap>,
    incoming_edges: RwLock<RoaringBitmap>,
}

pub struct RwLockedGraph {
    nodes: RwLock<HashMap<u32, RwLockedNodeMap>>,
    pub is_loaded: RwLock<bool>,
}

impl RwLockedGraph {
    pub fn new(expected_node_count: u32) -> Self {
        RwLockedGraph {
            nodes: RwLock::new(HashMap::with_capacity(expected_node_count as usize)),
            is_loaded: RwLock::new(false),
        }
    }

    /// Adds an edge between a given source and target node.
    pub fn add_edge(&self, source: u32, target: u32) {
        let mut nodes = self.nodes.write().unwrap();
        let source_map = nodes.entry(source).or_insert_with(|| RwLockedNodeMap {
            outgoing_edges: RwLock::new(RoaringBitmap::new()),
            incoming_edges: RwLock::new(RoaringBitmap::new()),
        });
        source_map.outgoing_edges.write().unwrap().insert(target);

        let target_map = nodes.entry(target).or_insert_with(|| RwLockedNodeMap {
            outgoing_edges: RwLock::new(RoaringBitmap::new()),
            incoming_edges: RwLock::new(RoaringBitmap::new()),
        });
        target_map.incoming_edges.write().unwrap().insert(source);
    }

    /// Removes the edge between a given source and target node.
    pub fn remove_edge(&self, source: u32, target: u32) {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(source_map) = nodes.get_mut(&source) {
            source_map.outgoing_edges.write().unwrap().remove(target);
        }

        if let Some(target_map) = nodes.get_mut(&target) {
            target_map.incoming_edges.write().unwrap().remove(source);
        }

        // TODO
        // Create a bitmap for updated edges that can be written to disk asynchronously. 
    }

    /// Returns the incoming_edges for a target node.
    pub fn get_incoming_edges(&self, target: u32) -> RoaringBitmap {
        if let Some(node) = self.nodes.read().unwrap().get(&target) {
            node.incoming_edges.read().unwrap().clone()
        } else {
            RoaringBitmap::new()
        }
    }

    /// Returns the outgoing_edges for a source node.
    pub fn get_outgoing_edges(&self, source: u32) -> RoaringBitmap {
        if let Some(node) = self.nodes.read().unwrap().get(&source) {
            node.outgoing_edges.read().unwrap().clone()
        } else {
            RoaringBitmap::new()
        }
    }

    /// Checks if the outgoing_edges of a source node contain a target node.
    pub fn has_edge(&self, source: u32, target: u32) -> bool {
        self.get_outgoing_edges(source).contains(target)
    }

    /// Checks that a node exists.
    pub fn get_node(&self, source: u32) -> Option<u32> {
        Some(source).filter(|&s| self.nodes.read().unwrap().contains_key(&s))
    }

    /// Loads from a TSV file given a path.
    pub fn load_from_csv(&mut self, path: &str, delimiter: Option<u8>) -> std::io::Result<()> {
        let file = File::open(path)?;
        let mut reader_builder = ReaderBuilder::new();
           

        // sets delimeter if provided
        let reader_builder = match delimiter {
            Some(delim) => reader_builder.delimiter(delim),
            None => &mut reader_builder,
        };

        let mut rows = reader_builder.has_headers(false).from_reader(BufReader::new(file));
        let mut row_count = 0;
            
        for res in rows.records() {
            row_count += 1;

            if row_count % 100_000 == 0 {
                info!("loaded {} rows to raphle instance", row_count);
            }

            let rec = res?;
            
            let source: u32 = rec.get(0)
                .unwrap()
                .parse::<u32>()
                .unwrap();
            let target: u32 = rec.get(1)
                .unwrap()
                .parse::<u32>()
                .unwrap();

            self.add_edge(source, target);
        }

        *self.is_loaded.write().unwrap() = true;
        info!("Loaded graph with {} edges", row_count); // should be user count

        Ok(())
    }
}
