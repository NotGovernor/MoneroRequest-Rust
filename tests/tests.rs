#![allow(non_snake_case)]
use MoneroRequest_Rust;

#[test]
fn Test_GenRandomPaymentID() {
	let Output = MoneroRequest_Rust::GenRandomPaymentID();
	
	assert_eq!(Output.len(), 16);
	assert!(Output.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')), "Invalid hex character found.");
}