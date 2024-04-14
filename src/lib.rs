/*! # MoneroRequest_Rust Crate
 
MoneroRequest_Rust is an easy way to decode/encode monero-request codes in Rust.

Use [EncodePaymentRequest] and [DecodePaymentRequest] to create and decode requests. 
[GenRandomPaymentID] is a helper function for generating random PaymentIDs valid to the 
Monero protocol standard.

You may review the Monero Payment Request Standard [here](https://github.com/lukeprofits/Monero_Payment_Request_Standard).*/
#![allow(non_snake_case)]
use std::{io::Write, iter};
use rand::prelude::*;
use base64::Engine;
use serde::{Deserialize, Serialize};
use flate2::{write::GzEncoder, Compression};
use chrono::{prelude::*};



#[derive(Serialize, Deserialize, Debug)]
pub struct MoneroRequest {
	pub CustomLabel: String,
	pub SellersWallet: String,
	pub Currency: String,
	pub Amount: String,
	pub PaymentID: String,
	pub StartDate: String,
	pub DaysPerBillingCycle: u8,
	pub NumberOfPayments: u8,
	pub ChangeIndicatorURL: String,
	pub Version: String
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
		if self.StartDate.len() == 0 {
			self.StartDate = Utc::now().to_string();
		} else {
			self.StartDate = match self.StartDate.parse::<DateTime<Utc>>() {
				Ok(r) => r.to_string(),
				Err(e) => return Err(MoneroRequestError::ChronoError(e))
			}
		}

		// Currency
		if !vec!["USD", "XMR"].contains(&self.Currency.as_str()) { return Err(MoneroRequestError::InvalidInput("Invalid Currency.")); }

		// Amount
		if !regex::Regex::new(r"(?m)[\d,.]+").unwrap().is_match(&self.Amount) {
			return Err(MoneroRequestError::InvalidInput("Invalid Amount."));
		};

		// Days per billing cycle
		if self.DaysPerBillingCycle == 0 { return Err(MoneroRequestError::InvalidInput("DaysPerBillingCycle cannot be zero.")) }

		// NumberOfPayments
		// Any u8 will be valid.

		// ChangeIndicatorURL
		if self.ChangeIndicatorURL.len() > 0 {
			match url::Url::parse(&self.ChangeIndicatorURL) {
				Err(e) => return Err(MoneroRequestError::UrlError(e)),
				Ok(r) => if r.cannot_be_a_base() { return Err(MoneroRequestError::InvalidInput("ChangeIndicatorURL is an invalid URL.")); }
			};
		}

		// Version
		match self.Version.as_str() {
			"" => self.Version = "1".to_string(),
			"1" => {},
			_ => return Err(MoneroRequestError::InvalidInput("Unsupported version."))
		}

		Ok(())
	}
}



pub fn DecodePaymentRequest(Request: String) {
	// TODO
}


/// Accepts a [`MoneroRequest`] struct, validates it, and outputs a String constituting a valid Monero Payment Request
///
/// If no [`PaymentID`](MoneroRequest::PaymentID) is provided, one will be generated randomly using [`GenRandomPaymentID`]. If you explicitely
/// do not want to have a PaymentID you may set it to `0000000000000000` (16 zeroes).
///
/// If no [`CustomLabel`](MoneroRequest::CustomLabel) is provided `Monero Payment Request` will be used.
///
/// [`DaysPerBillingCycle`](MoneroRequest::DaysPerBillingCycle) may not be zero.
///
/// [`NumberOfPayments`](MoneroRequest::NumberOfPayments) may not be zero.
///
/// If no [`StartDate`](MoneroRequest::StartDate) is provided, now will be used. StartDate should be either in UTC time or contain enough 
/// information to be parsed into UTC time. The Chrono library is used for this parsing. To guarentee compatibility 
/// with this MoneroRequest library use Chrono's DateTime type.
///
/// If [`Version`](MoneroRequest::Version) is blank the latest version will be used.
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



/// Generates a random string that may be used as a Monero protocol payment_id in an integrated address.
///
/// Returns 16 hex characters as a string. Example: `60B6A010501201F1`
///
/// This will be used by default if no [`PaymentID`](MoneroRequest::PaymentID) is provided when encoding a 
/// new request with [`EncodePaymentRequest`]. You may want to use this directly before calling [`EncodePaymentRequest`] 
/// so you can check it for uniqueness against prior transactions in your records.
pub fn GenRandomPaymentID() -> String {
	let mut RNG = rand::thread_rng();
	let HEXChars = "0123456789abcdef";

	let Output: String = iter::repeat_with(|| HEXChars.chars().nth(RNG.gen_range(0..=15)).unwrap()).take(16).collect();
	
	return Output;
}



/// Enum containing all errors we may emit.
#[derive(thiserror::Error, Debug)]
pub enum MoneroRequestError {
	#[error("{0}")]
	InvalidInput(&'static str),

	#[error(transparent)]
	SerdeError(#[from] serde_json::Error),

	#[error(transparent)]
	ChronoError(#[from] chrono::ParseError),

	#[error(transparent)]
	UrlError(#[from] url::ParseError),

	#[error("{0}")]
	GZipError(&'static str)
}









