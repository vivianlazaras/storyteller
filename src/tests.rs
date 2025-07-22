#[macro_export]
macro_rules! generate_post_integration_test {
    (
        $test_name:ident,
        $route:expr,
        $form_ty:ty
    ) => {
        #[test]
        fn $test_name() {
            use rocket::http::ContentType;
            use rocket::local::blocking::Client;
            use std::time::{Duration, Instant};

            let rocket = crate::rocket(); // Your Rocket instance builder
            let client = Client::tracked(rocket).expect("valid rocket instance");

            // Create test form data
            let form_data: $form_ty = <$form_ty>::test();

            // Serialize form data to x-www-form-urlencoded string
            let body = serde_urlencoded::to_string(&form_data)
                .expect("Failed to serialize form data to urlencoded");

            // Send POST request with form urlencoded content type
            let response = client
                .post($route)
                .header(ContentType::Form)
                .body(body)
                .dispatch();

            assert!(
                response.status().class().is_success(),
                "Response was not successful"
            );

            let resp_body = response.into_string().expect("Response body");
        }
    };
}

generate_post_integration_test!(create_fragment_test, "/fragments/", FragmentForm);
