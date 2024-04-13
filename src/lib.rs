/*! # MoneroRequest-Rust Crate
 
`MoneroRequest-Rust` is an easy way to decode/encode monero-request codes in Rust.*/
#![allow(non_snake_case)]
use std::{io::Write, iter};
use rand::prelude::*;
use base64::Engine;
use serde::{Deserialize, Serialize};
use flate2::{write::GzEncoder, Compression};




#[derive(Serialize, Deserialize, Debug)]
pub struct MoneroRequest {
	CustomLabel: String,
	SellersWallet: String,
	Currency: String,
	Amount: String,
	PaymentID: String,
	StartDate: String,
	DaysPerBillingCycle: u8,
	NumberOfPayments: u8,
	ChangeIndicatorURL: String,
	Version: String
}

impl MoneroRequest {
	fn Validate(&mut self) -> Result<(), MoneroRequestError> {
		// Label
		if self.CustomLabel.len() == 0 { self.CustomLabel = "Monero Payment Request".to_string() }

		// Seller wallet address
		match self.SellersWallet.len() {
			0 => return Err(MoneroRequestError::InvalidInput("No seller wallet specified.")),
			95 | 106 => {},
			_ => return Err(MoneroRequestError::InvalidInput("Incorrect seller wallet address length."))
		}

		let FirstChar = self.SellersWallet.chars().next().unwrap();	// This unwrap should be safe as we checked lenth > 0 earlier
		if FirstChar != '4' && FirstChar != '8' {
			return Err(MoneroRequestError::InvalidInput("Invalid wallet address. Doesnt start with 4 or 8."));
		}

		for C in self.SellersWallet.chars() {
			match C {
				'0'..='9' | 'A'..='Z' | 'a'..='z' => {},
				_ => return Err(MoneroRequestError::InvalidInput("Invalid character in wallet address."))
			}
		}

		// PaymentID
		match self.PaymentID.len() {
			0 => self.PaymentID = GenRandomPaymentID(),
			1..=15 | 17.. => return Err(MoneroRequestError::InvalidInput("Invalid PaymentID length.")),
			16 => {}
		}

		for C in self.PaymentID.chars() {
			match C {
				'0'..='9' | 'a'..='f' => {}
				_ => return Err(MoneroRequestError::InvalidInput("Invalid character in PaymentID."))
			}
		}

		// StartDate
		// todo!

		// Amount
		// todo!

		// Days per billing cycle
		if self.DaysPerBillingCycle == 0 { return Err(MoneroRequestError::InvalidInput("DaysPerBillingCycle cannot be zero.")) }

		// NumberOfPayments
		if self.NumberOfPayments == 0 { return Err(MoneroRequestError::InvalidInput("NumberOfPayments cannot be zero.")) }

		// ChangeIndicatorURL
		// todo!

		// Version
		if self.Version != "1" { return Err(MoneroRequestError::InvalidInput("Unsupported version.")) }




		Ok(())
	}
}



pub fn DecodePaymentRequest(Request: String) {
	// TODO
}


/// Accepts a MoneroRequest struct, validates it, and outputs a String constituting a Monero Payment Request
pub fn EncodePaymentRequest(mut Request: MoneroRequest) -> Result<String, MoneroRequestError> {
	// Validate input
	if let Err(e) = Request.Validate() { return Err(e); }

	// Serialize to JSON
	let Output = match serde_json::to_string(&Request) {
		Ok(o) => o,
		Err(e) => return Err(MoneroRequestError::SerdeError(e))
	};

	// GZip the output
	let mut GZipOutput = GzEncoder::new(Vec::new(), Compression::default());
	if GZipOutput.write_all(Output.as_bytes()).is_err() { return Err(MoneroRequestError::GZipError("Error compressing data."))}
	let GZipOutput = match GZipOutput.finish() {
		Ok(r) => r,
		Err(_) => return Err(MoneroRequestError::GZipError("Error compressing data."))
	};

	// Base64 the output -- This cannot fail? Sus.
	let Output = base64::engine::general_purpose::STANDARD.encode(GZipOutput);

	// Add tags
	let Output = format!("monero-request:1:{Output}");

	return Ok(Output);
}



/// Generates a random string that may be used as a Monero payment_id in an integrated address.
///
/// Returns 16 hex characters as a string. Example: 60B6A010501201F1
pub fn GenRandomPaymentID() -> String {
	let mut RNG = rand::thread_rng();
	let HEXChars = "0123456789abcdef";

	let Output: String = iter::repeat_with(|| HEXChars.chars().nth(RNG.gen_range(0..=15)).unwrap()).take(16).collect();
	
	return Output;
}






/// All errors we may emit
#[derive(thiserror::Error, Debug)]
pub enum MoneroRequestError {
	#[error("{0}")]
	InvalidInput(&'static str),

	#[error(transparent)]
	SerdeError(#[from] serde_json::Error),

	#[error("{0}")]
	GZipError(&'static str)
}









