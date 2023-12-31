//Sources:
//https://river.com/learn/terms/m/merkle-sum-tree/
//https://www.youtube.com/watch?v=_RfxLg6K9oE&list=PLVvjrrRCBy2JSHf9tGxGKJ-bYAN_uDCUL&index=2
//https://www.geeksforgeeks.org/introduction-to-merkle-tree/#:~:text=Merkle%20tree%20also%20known%20as,as%20far%20left%20as%20possible.
//https://chat.openai.com/

use sha2::{Digest, Sha256};

pub trait SumCommitment {
    fn amount(&self) -> u64; //returns unsigned 64 bit integer.
    fn digest(&self) -> [u8; 32]; //returns array of 32 items of type unsigned 8 bits i.e 32 bytes.
}

pub struct BalanceCommitment { //Struct to implement SumCommitment that will act as a node in the Merkle Sum Tree.
    amount: u64, 
    hash: [u8; 32]
}

impl BalanceCommitment { //Implementing helper function for BalanceCommitment.
    pub fn new(amount: u64, hash: [u8; 32]) -> Self { //Function to intialize balance commitment instance.
        BalanceCommitment { amount, hash }
    }
}

impl SumCommitment for BalanceCommitment { //Implement the SumCommitment trait for BalanceCommitment struct.
    fn amount(&self) -> u64 { //Function to return the amount in the BalanceCommitment.
        self.amount
    }

    fn digest(&self) -> [u8; 32] { //Function to return the hash in the BalanceCommitment
        self.hash
    }
}

impl Clone for BalanceCommitment { //Implement Clone trait for BalanceCommitment struct to use function clone.
    fn clone(&self) -> Self {
        BalanceCommitment {
            amount: self.amount,
            hash: self.hash
        }
    }
}

pub trait ExclusiveAllotmentProof<C: SumCommitment> {
    fn position(&self) -> usize; //returns an int
    fn sibling(&self, height: u8) -> Option<C>; //returns object balance commitment
    fn verify(&self, root_commitment: &C) -> bool; //returns boolean
}

pub struct AllotmentProof { //Struct to implement ExclusiveAllotmentProof.
    position: usize,
    proof_path: Vec<BalanceCommitment>,
    node: BalanceCommitment
}

impl AllotmentProof { //Implementing helper function for AllotmentProof.
    pub fn new( position: usize, proof_path: Vec<BalanceCommitment>, node: BalanceCommitment) -> Self { //Function to intialize Allotment Proof instance.
        AllotmentProof { position, proof_path, node}
    }
}

impl ExclusiveAllotmentProof<BalanceCommitment> for AllotmentProof { //Implement the ExclusiveAllotmentProof trait for AllotmentProof struct.
    fn position(&self) -> usize { //Function to return the position of the node to be proven.
        self.position
    }

    fn sibling(&self, height: u8) -> Option<BalanceCommitment> { //Function to return the sibling of the node to be proven at a given height in the tree.
        if height < self.proof_path.len() as u8 {
            Some(self.proof_path[height as usize].clone())
        } else {
            None
        }
    }

    fn verify(&self, root_commitment: &BalanceCommitment) -> bool { //Function to verify that the proof generated by alice will be equal to the root commitmeny.
        let mut current_commitment = self.node.clone(); //current node to be proven.

        // Traverse the proof path to compute the calculated root commitment
        for height in 0..self.proof_path.len() {
        // Retrieve the sibling commitment at the specified height or throw an error if not found.
            let sibling_commitment = self.sibling(height as u8).unwrap_or_else(|| {
                panic!("Invalid height in proof path during verification");
            });

            // Determine whether to concatenate current or sibling first based on the position of the node, the logic is kinda confusing.
            let bit = (self.position >> height) & 1;

            let (left_commitment, right_commitment) = if bit == 0 {
                (current_commitment.clone(), sibling_commitment.clone())
            } else {
                (sibling_commitment.clone(), current_commitment.clone())
            };

            // Combine left commitment and right commitment amount and hash.
            let parent_amount = left_commitment.amount() + right_commitment.amount();
            let parent_hash = hash_bytes(&merge_u8_arrays(
                left_commitment.digest(),
                right_commitment.digest(),
            ));

            current_commitment = BalanceCommitment::new(parent_amount, parent_hash);
        }

        // Check if the calculated root commitment amount matches the provided root commitment amount.
        if current_commitment.amount() == root_commitment.amount() {
            // Check if the calculated root commitment hash matches the provided root commitment hash.
            let current_digest = current_commitment.digest();
            let root_digest = root_commitment.digest();

            for i in 0..32 {
                if current_digest[i] != root_digest[i] {
                    return false; // Hashes do not match.
                }
            }

            return true; // Hashes and amounts match.
        } else {
            false  // amounts do not match.
        }
    }
}

pub trait MerkleTree<C: SumCommitment, P: ExclusiveAllotmentProof<C>> {
    fn new(values: Vec<u64>) -> Self; //returns object Merkle  Sum Tree.
    fn commit(&self) -> C; //returns object of type balance commitment.
    fn prove(&self, position: usize) -> P; //returns object of type AllotmentProof.
}

pub struct MerkleSumTree { //Struct to implement MerkleTree.
    data: Vec<BalanceCommitment>,
}

impl MerkleSumTree { //Implementing helper functions for MerkleSumTree.
    fn init(&mut self, node: usize, start: usize, end: usize, input: &Vec<BalanceCommitment>) {//Function that helps create the Merkle Sum Tree.
     if start == end { //In case we reach a leaf.
        self.data[node] = input[start].clone();
    } else {
        let mid = (start + end) / 2;

        // Recursively initialize the left and right subtrees.
        self.init(node * 2 + 1, start, mid, input); 
        self.init(node * 2 + 2, mid + 1, end, input); 

        // Concatenate the left and right hash values to create the current node's hash.
        let left_hash = self.data[node * 2 + 1].digest();
        let right_hash = self.data[node * 2 + 2].digest();
        let concatenated_hash = merge_u8_arrays(left_hash, right_hash);

        // Set the current node's hash and amount based on the concatenated values.
        self.data[node].hash = hash_bytes(&concatenated_hash);
        self.data[node].amount = self.data[node * 2 + 1].amount + self.data[node * 2 + 2].amount;
        }
    }

    fn print_tree_balances(&self, node: usize, depth: usize) { //Function to print the tree balances.
        if node < self.data.len() {
            self.print_tree_balances(node * 2 + 2, depth + 1);
            println!("{}Amount: {}", "  ".repeat(depth), self.data[node].amount());
            self.print_tree_balances(node * 2 + 1, depth + 1);
        }
    }

    fn print_tree_hashes(&self, node: usize, depth: usize) { //Function to print the tree hashes.
        if node < self.data.len() {
            self.print_tree_hashes(node * 2 + 2, depth + 1);
            println!("{}Hash: {:x?}", "  ".repeat(depth), &self.data[node].digest());
            self.print_tree_hashes(node * 2 + 1, depth + 1);
        }
    }

}

impl MerkleTree<BalanceCommitment, AllotmentProof> for MerkleSumTree { //Implement the MerkleTree trait for MerkleSumTree struct.
    fn new(input: Vec<u64>) -> Self { //Function to intialize Merkle Sum Tree  instance.
        // Check if the size of the input is a power of 2
        if input.len() & (input.len() - 1) != 0 {
            panic!("Input size must be a power of 2");
        }

        let total: u64 = input.iter().sum();
        // Check if the sum of the inputs does not exceed 1,000,000,000
        if total > 1_000_000_000 {
            panic!("Sum of input values exceeds 1,000,000,000");
        }

        //create array of balance commitments.
        let balance_commitments: Vec<BalanceCommitment> = input
            .iter()
            .map(|&amount| {
                let hash_result = hash_bytes(int_to_u8_slice(amount));
                BalanceCommitment::new(amount, hash_result)
            })
            .collect();

        // Create a new MerkleSumTree with an initial data structure to store balance commitments,
        let mut tree = MerkleSumTree {
            data: vec![BalanceCommitment::new(0, [0; 32]); input.len() * 2 - 1],
        };
        tree.init(0, 0, input.len() - 1, &balance_commitments);//helper function
        tree
    }

    fn commit(&self) -> BalanceCommitment { //Function to return the root of the Merkle Sum Tree.
        self.data[0].clone()
    }

    fn prove(&self, position: usize) -> AllotmentProof { //Function to return the proof for the node that its index was provided.
        // Check if the position is valid
        let len = (self.data.len() + 1) / 2;
        if position + len - 1 >= self.data.len() {
            panic!("Invalid position for proof");
        }

        // Initialize variables for proof generation
        let mut proof_path: Vec<BalanceCommitment> = Vec::new();
        let mut current_pos = position + len - 1;
        let current_node = self.data[position + len - 1].clone();

        // Traverse the path from the leaf to the root
        while current_pos > 0 {
            let sibling_pos = if current_pos % 2 == 0 {
                current_pos - 1
            } else {
                current_pos + 1
            };

            // Calculate the parent position
            let parent_pos = (current_pos - 1) / 2;

            let sibling_commitment = self.data[sibling_pos].clone();
            proof_path.push(sibling_commitment);
            current_pos = parent_pos;
        }

        // Create the proof 
        AllotmentProof::new(position, proof_path, current_node)
    }
}

fn hash_bytes(slice: &[u8]) -> [u8; 32] { //Function that returns array of 32 items of type unsigned 8 bits i.e 32 bytes.
    let mut hasher = Sha256::new();
    hasher.update(slice);
    hasher.finalize().into()
}

fn int_to_u8_slice(number: u64) -> &'static [u8; 8] { //Function that turns u64 numbers to u8 array to be used in the hash function.
    Box::leak(Box::new(number.to_be_bytes()))
}

fn merge_u8_arrays(arr1: [u8; 32], arr2: [u8; 32]) -> [u8; 64] { //Function that concatinates 2 hashes into one.
    let mut merged = [0; 64];

    for i in 0..32 {
        merged[i] = arr1[i];
        merged[i + 32] = arr2[i];
    }

    merged
}

fn print_proof_path(proof: &AllotmentProof) { //Function that prints the proof structure.
    println!("Proof Position: {}", proof.position());
    println!("Proof Path:");

    for (height, commitment) in proof.proof_path.iter().enumerate() {
        println!("  Height {}: {:?}", height, commitment.amount);
    }
}

fn main() {
    //Some test data
    let number: u64 = 300;

    let hash_result = hash_bytes(int_to_u8_slice(number));
    
    println!("Hash of '{}' is: {:x?}", number, hash_result);

    let balance_commitment = BalanceCommitment::new(number, hash_result);

    let input_values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let sum_tree = MerkleSumTree::new(input_values);

    println!("Tree Representation of the Sum Tree (Amounts):");
    sum_tree.print_tree_balances(0, 0);

    println!("Tree Representation of the Sum Tree (Hashes):");
    sum_tree.print_tree_hashes(0, 0);

    println!("Data length : {}", sum_tree.data.len());

    let root_commitment = sum_tree.commit();

    println!("Root Commitment:");
    println!("Amount: {}", root_commitment.amount());
    println!("Hash: {:x?}", root_commitment.digest());
    
    let proof = sum_tree.prove(3);
    print_proof_path(&proof);
    
    let sibling_commitment = proof.sibling(1).expect("Sibling not found");
    println!("Amount of Sibling Commitment at Height {}: {}", 1, sibling_commitment.amount());
    
    println!("Is Valid ? : {}", proof.verify(&root_commitment));
}
