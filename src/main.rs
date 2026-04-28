pub mod model;
pub mod database;
pub mod handlers;
pub mod parse;

use crate::database::Database;
use axum::{extract::State, Error, Router};
use axum::routing::get;
use handlers::addItem;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use crate::model::{Coffee, Roast, ItemOrder, CustomerOrder, Size, Inventory};
use std::collections::HashMap;


//shared state struct for info/structures needed by the app

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
    pub carts : Arc<Mutex<HashMap<u32, CustomerOrder>>>,

    
    //using Arc and Mutex so that the multiple users(threads) can access this data safely
     
    pub inventory : Arc<Mutex<Inventory>>,

    pub db : Arc<Mutex<Database>>

}


#[tokio::main]
async fn main() {
    //first, initialize database:
    let db = match db_init() {
        Ok(db)=> db,
        Err(e)=> {
            eprintln!("Failed to initialize database: {e}");
            return ;//Redirect::to("/error");
        }
        
    };
    
    //initalize shared state here
    let state = AppState{
        carts : Arc::new(Mutex::new(HashMap::new())),


        inventory  :Arc::new(Mutex::new(Inventory::new())),

        db : Arc::new(Mutex::new(db)),
    };

    //let query 

    //here is where we will assign our handlers to the route

    let app = Router::new()
        .route("/add", axum::routing::get(addItem))
        .with_state(state);


    let listener = tokio::net::TcpListener::bind("localhost:7008")
        .await
        .expect("failed to bind to port");

    //serve requests to users
    axum::serve(listener, app)
        .await
        .expect("server failed to start")

    /*
    //doing some testing of the code in model

    println!("--- Initializing Shop Inventory ---");
    let mut shop_inventory = Inventory::new();


    //adding inventory
    shop_inventory.add_stock(Coffee::Columbian, 50);
    shop_inventory.add_stock(Coffee::MidnightRoast, 20);
    //shop_inventory.print_inventory();


    //simulating customer getting to page
    println!("\n--- Creating New Customer Order ---");
    let mut current_session_order = CustomerOrder::new(101, 5001);


   
    // Link: /add?coffee=Arabica&size=Large&qty=2, user clicks this link
    println!("Adding 2 Large Arabica...");
    current_session_order.add_item(Coffee::Arabica, Roast::Medium, Size::Large, 2.0);
    shop_inventory.reduce_stock(Coffee::Arabica, 2);


    println!("\n--- Final Receipt ---");
    println!("Order ID: {}", current_session_order.id);
    println!("Customer ID: {}", current_session_order.customer_id);


    for item in &current_session_order.items {
        println!(
            "Item: {:?} ({:?}) | Size: {:?} | Qty: {} | Subtotal: ${:.2}",
            item.coffee, item.roast.to_str(), item.size.to_str(), item.quantity, item.price
        );
    }

    println!("---------------------------");
    println!("TOTAL ORDER PRICE: ${:.2}", current_session_order.total_price);
    println!("---------------------------");

    */

}

//this function makes sure our database is initialized with empty tables ready to go.
pub fn db_init() -> Result<Database, rusqlite::Error>{
    let db = match Database::new("coffee.db".to_string()){
        Ok(db) => {
            println!("Successfully connected to {}",{&db.name});
            db
        },
        Err(e) => {
            println!("Error connecting to database: {e}");
            return Err(e);
        }
    };

    match db.drop_tables(){
        Ok(a) => println!("Tables dropped successfully"),
        Err(e) => println!("Error dropping tables: {e}")
    };

    match db.create_tables(){
        Ok(a) => println!("Tables created successfully"),
        Err(e) => println!("Error creating tables: {e}")
    };

    Ok(db)
}

