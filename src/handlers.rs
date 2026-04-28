/*
here is where we will build the functionality for handling the different url
probably will have something like 

/add_inventory
/add
/order

definitely a couple others but this is the idea
*/

use axum::extract::{State, Query};
use axum::response::{Html, Redirect};
use serde::Deserialize;
use rand::random;

use crate::AppState;
use crate::model::{CustomerOrder, Roast};
//use crate::parse::{parse_coffee, parse_size};



/*
these struct will allow us to handle the parameters we want by passingh them when we use a handler
this will be used for example when using the basic get/

will be passing these and the Appstate so we have access to cart, inventory and db
*/
#[derive(Deserialize)]
pub struct PageParameters{
    pub cart_id : Option<u32>
}

#[derive(Deserialize)]
pub struct AddOrderParams{
    pub cart_id : u32,
    pub coffee : String,
    pub size : String,
    pub quantity : f32
}


pub struct AddInventoryParams{
    pub cart_id : Option<u32>
}


pub struct CheckoutParams{
    pub cart_id : u32
}

/*
gonna try to build a handler for when the user clicks a add item to order
before doing all the setup of the actual page
*/


pub async fn addItem(State(state) : State<AppState>, Query(params) : Query<AddOrderParams>) -> Redirect{

    let coffee = &params.coffee;
    let size = &params.size;

    let amount = &params.quantity;

    //believe using .lock() here is right because we dont want multiple threads editing
    let mut carts = state.carts.lock().unwrap();
    let mut inventory = state.inventory.lock().unwrap();


    //finding the entry in the hashmap and if it doesnt exist we insert the a customer order with the current cart_id
    let cart = carts
        .entry(params.cart_id)
        .or_insert(||CustomerOrder::new(params.cart_id as i32));


    if inventory.reduce_stock(coffee, amount){
        cart.add_item(coffee, Roast::Medium, size, amount);
    }  

    Redirect::to(&format!("/?cart_id={}", params.cart_id)) 
}