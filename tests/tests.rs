#![allow(non_snake_case)]
use MoneroRequest_Rust::{self, MoneroRequest};

#[test]
fn Test_GenRandomPaymentID() {
	let Output = MoneroRequest_Rust::GenRandomPaymentID();
	
	assert_eq!(Output.len(), 16);
	assert!(Output.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')), "Invalid hex character found.");
}

#[test]
fn Test_EncodePaymentRequest() {
	let Input = MoneroRequest {
		CustomLabel: "A label".to_string(),
		SellersWallet: "4At3X5rvVypTofgmueN9s9QtrzdRe5BueFrskAZi17BoYbhzysozzoMFB6zWnTKdGC6AxEAbEE5czFR3hbEEJbsm4hCeX2S".to_string(),
		Currency: "USD".to_string(),
		Amount: "123.45".to_string(),
		PaymentID: "".to_string(),
		StartDate: "2023-04-26T13:45:33.123Z".to_string(),
		DaysPerBillingCycle: 30,
		NumberOfPayments: 12,
		ChangeIndicatorURL: "https://Somewhere.com".to_string(),
		Version: "1".to_string()
	};

	let Output = MoneroRequest_Rust::EncodePaymentRequest(Input);

	assert!(Output.is_ok(), "Error encoding request: {:?}", Output.unwrap_err());

	let Output = Output.unwrap();

	assert!(Output == "Some string");
}