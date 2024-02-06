use tfhe::shortint::prelude::*;

// key is the ServerKey instance used for homomorphic computation.
// target is the ciphertext corresponding to the item code being queried.
// inventory is a plaintext list of all (item, amount) pairs present in the inventory

fn query(key: ServerKey, mut target: Ciphertext, inventory: &[(u8, u8)]) -> Ciphertext {
///Make an variable for the sum
  let mut final_amount_in_stock =key.unchecked_scalar_mul(&mut target, 0);
  for (item_code, amount) in inventory {
    let mut item_code_encrypted = key.unchecked_scalar_mul(&mut target, 0);
    key.unchecked_scalar_add_assign(&mut item_code_encrypted, *item_code);
    //Compare tagret with item code
    let comparisonResult = key.unchecked_equal(&target, &item_code_encrypted);
    //multiply comparison result with amount
    let multiplyingResult = key.unchecked_scalar_mul(&comparisonResult, *amount);
    //add results together
    key.unchecked_add_assign(&mut final_amount_in_stock,& multiplyingResult); 
  }
  final_amount_in_stock
}
//multuply   key.unchecked_scalar_mul_assign(&mut target, scalar);
//add        key.unchecked_scalar_add_assign(&mut target, scalar);
//comparing 2 encrypted values key.unchecked_equal(&mut target, &mut final_amount_in_stock);
//assign keyword automatically assign the result to the target
//scalar keyword is used to do the operation to one encypted value with one scalar

fn main() {
  println!("HI");
}


#[cfg(test)]
mod tests {
  use tfhe::shortint::prelude::*;
  use tfhe::shortint::parameters::PARAM_MESSAGE_4_CARRY_0_KS_PBS;

  use crate::query;
  
  #[test]
  fn test_it() {
    let (client_key, server_key) = gen_keys(PARAM_MESSAGE_4_CARRY_0_KS_PBS);

    let item_code = 0u8;

    let item_code_ciphertext = client_key.encrypt(item_code as u64);

    let stock_ciphertext = query(server_key, item_code_ciphertext, &[
      (1, 2),
      (2, 1)
    ]);

    let stock_count = client_key.decrypt(&stock_ciphertext);

    assert_eq!(stock_count, 0);
  }
}
