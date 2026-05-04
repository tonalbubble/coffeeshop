pub mod model;
pub mod database;
pub mod handlers;
pub mod parse;

use crate::model::{CustomerOrder, Inventory};
use crate::database::{Database, db_init};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use axum::Router;
use handlers::{add_item, load_page, admin_page, checkout, confirm_checkout, add_inventory};

/*
    Arc allows multiple threads to safely own data, we need this as we are going to just have a simple implementation
    of a coffeeshop
    the idea of this struct though is so we dont have to handle each request for the data independently, because 
    we have it here all the users/threads will be able to access what they need in one place

    so this will store the global 'carts' being used and store inventory and the db

    not the best OO or threadsafe but it serves the purpose

*/
#[derive(Clone)]
pub struct AppState{
    //use a hashmap to match cart to customer so we dont just hav eone cart
    pub carts : Arc<Mutex<HashMap<i32, CustomerOrder>>>,

    //using Arc and Mutex so that the multiple users(threads) can access this data safely
    pub inventory : Arc<Mutex<Inventory>>,
    pub db : Arc<Mutex<Database>>,
    pub num_orders: Arc<Mutex<i32>>
}

#[tokio::main]
async fn main() {
    //initialize database
    let db = match db_init(false) {
        Ok(db)=> db,
        Err(e)=> {
            println!("Failed to initialize database: {e}");
            return ;
        }
    };
    //TODO get last id from database and set num_orders equal to that 
    //initalize shared state here
    let state = AppState{
        carts : Arc::new(Mutex::new(HashMap::new())),
        inventory  :Arc::new(Mutex::new(Inventory::new())),
        db : Arc::new(Mutex::new(db)),
        num_orders: Arc::new(Mutex::new(1))
    };


    //assign our handlers to the route
    let app = Router::new()
        .route("/", axum::routing::get(load_page))
        .route("/admin", axum::routing::get(admin_page))
        .route("/add", axum::routing::get(add_item))
        .route("/checkout", axum::routing::get(checkout))
        .route("/confirm_checkout", axum::routing::get(confirm_checkout))
        .route("/inventory/add",axum::routing::get(add_inventory))
        .with_state(state);

    //initialize tokio listener
    let listener = tokio::net::TcpListener::bind("localhost:7008")
        .await
        .expect("failed to bind to port");

    //serve requests to users
    axum::serve(listener, app)
        .await
        .expect("server failed to start")
}