mod models; // Private to api_utils
mod urls;
pub mod api_handler; // Public to main.rs for access to the api handler struct that uses models
                     //  under the hood
