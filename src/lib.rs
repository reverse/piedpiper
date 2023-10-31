use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Seek};
use std::path::Path;
use std::collections::HashMap;
use std::error::Error;
use bit_vec::BitVec;

type EncoderResult<T> = Result<T, Box<dyn Error>>;

pub struct Encoder {
    file: File,
    word_counts: HashMap<char, u32>,
    embeddings: HashMap<char, String>,
    root: Box<TreeNode>,
}

#[derive(Debug)]
struct TreeNode {
    value: u32, 
    // we use a box here because we don't actually want to infinitely create a size
    // we just want to store the size of tree node, so we will calculate that at compile time
    // rather than infitely looping looking for a struct of infinite size
    // we will know the true size at compile time, which box allows
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    character: Option<char>,
}

impl TreeNode {
    fn new(value: u32, 
            left: Option<Box<TreeNode>>, 
            right: Option<Box<TreeNode>>, 
            character: Option<char>) -> TreeNode {
        TreeNode {
            value,
            left,
            right,
            character,
        }
    }
}

impl Encoder {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .open(path)?;

        let map = HashMap::new();
        let embeddings = HashMap::new();

        Ok(Encoder{
            file: f,
            word_counts: map,
            embeddings,
            root: Box::new(TreeNode::new(0, None, None, None)),
        })
    }

    pub fn encode(&mut self) -> EncoderResult<BitVec> {
        let mut f = BufReader::new(&self.file);

        for line in f.lines() {
            for c in line?.chars() {
                self.word_counts.entry(c).
                    and_modify(|counter| *counter += 1).
                    or_insert(1);
            }
            self.word_counts.entry('\n').
                and_modify(|counter| *counter += 1).
                or_insert(1);
        }

        self.assemble_tree()?;

        let code_str = String::from("");
        Self::assign_codes(&mut self.embeddings, self.root.as_ref(), code_str);

        let mut encoder = BitVec::new();

        f = BufReader::new(&self.file);
        // rewind the file pointer
        f.rewind()?;

        for line in f.lines() {
            for c in line?.chars() {
                for embedded_char in self.embeddings.get(&c).unwrap().chars() {
                    encoder.push(embedded_char == '1');
                }
            }
            for embedded_char in self.embeddings.get(&'\n').unwrap().chars() {
                encoder.push(embedded_char == '1');
            }
        }

        Ok(encoder)
    }

    fn assemble_tree(&mut self) -> EncoderResult<()> {
        let mut tree: Vec<_> = self.word_counts
            .iter()
            .map(|entry| Box::new(TreeNode::new(*entry.1, None, None, Some(*entry.0))))
            .collect();

        while tree.len() > 1 {
            // sort in ascending order
            tree.sort_by(|a, b| b.value.cmp(&a.value));
            // get our 2 current smallest values
            let first_node = tree.pop().unwrap();
            let second_node = tree.pop().unwrap();

            // combine their values
            let combined_freq = first_node.value + second_node.value;
            // create a new node that stores their combined values
            let root = TreeNode::new(combined_freq, Some(first_node), Some(second_node), None);
            // push it into our tree
            tree.push(Box::new(root));
        }

        self.root = tree.pop().unwrap();

        Ok(())
    }

    fn assign_codes(embeddings: &mut HashMap<char, String>,
                    p: &TreeNode,
                    s: String) {
        if let Some(ch) = p.character {
            embeddings.insert(ch, s);
        } else {
            if let Some(ref l) = p.left {
                Self::assign_codes(embeddings, l, s.clone() + "0");
            }
            if let Some(ref r) = p.right {
                Self::assign_codes(embeddings, r, s.clone() + "1");
            }
        }
    }

    pub fn decode(&self, input: BitVec) -> EncoderResult<String> {
        let mut output = String::from("");
        let root = self.root
            .as_ref();

        let mut nodeptr = root;
        for c in input {
            if !c {
                if let Some(ref left) = nodeptr.left {
                    nodeptr = left;
                }
            } else {
                if let Some(ref right) = nodeptr.right {
                    nodeptr = right;
                }
            }
            if let Some(value) = nodeptr.character {
                output.push(value);
                nodeptr = &root;
            }
        }

        Ok(output)
    }
}


