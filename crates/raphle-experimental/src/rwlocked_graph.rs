use std::{fs::File, io::BufReader, sync::RwLock};
use hashbrown::HashMap;
use roaring::bitmap::RoaringBitmap;
use csv::ReaderBuilder;
use tracing::info;

pub struct RWLockedNodeMap {
    outgoing_edges: RwLock<RoaringBitmap>,
    incoming_edges: RwLock<RoaringBitmap>,
}

pub struct RWLockedGraph {
    nodes: RwLock<HashMap<u32, RWLockedNodeMap>>,
    pub is_loaded: RwLock<bool>,
}

impl RWLockedGraph {
    pub fn new(expected_node_count: u32) -> Self {
        RWLockedGraph {
            nodes: RwLock::new(HashMap::with_capacity(expected_node_count as usize)),
            is_loaded: RwLock::new(false),
        }
    }

    /// Adds an edge between a given source and target node.
    pub fn add_edge(&self, source: u32, target: u32) {
        let mut nodes = self.nodes.write().unwrap();
        let source_map = nodes.entry(source).or_insert_with(|| RWLockedNodeMap {
            outgoing_edges: RwLock::new(RoaringBitmap::new()),
            incoming_edges: RwLock::new(RoaringBitmap::new()),
        });
        source_map.outgoing_edges.write().unwrap().insert(target);

        let target_map = nodes.entry(target).or_insert_with(|| RWLockedNodeMap {
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
    pub fn load_from_tsv(&mut self, path: &str) -> std::io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut rows = ReaderBuilder::new().delimiter(b'\t').from_reader(reader);
        let mut rec = csv::ByteRecord::new();
        let mut row_count = 0;

        while rows.read_byte_record(&mut rec)? {
            row_count += 1;

            if row_count % 100_000 == 0 {
                info!("Processed {} rows", row_count);
            }

            let source: u32 = std::str::from_utf8(rec.get(0).unwrap())
                .unwrap()
                .parse()
                .unwrap();
            let target: u32 = std::str::from_utf8(rec.get(1).unwrap())
                .unwrap()
                .parse()
                .unwrap();

            self.add_edge(source, target);
        }
        
        *self.is_loaded.write().unwrap() = true;
        info!("Loaded graph with {} edges", row_count); // should be user count

        Ok(())
    }
}
