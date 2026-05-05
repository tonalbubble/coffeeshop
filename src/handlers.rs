use axum::extract::{State, Query};
use axum::response::{Html, Redirect};
use serde::Deserialize;

use crate::parse::{parse_coffee, parse_size};
use crate::AppState;
use crate::model::{CustomerOrder, Roast, Size};

//struct to hold parameters we want to pass to the load_page function
#[derive(Deserialize)]
pub struct PageParameters{
    pub cart_id : Option<i32>
}


//parameters needed for addOrder function
#[derive(Deserialize)]
pub struct AddOrderParams{
    pub cart_id : i32,
    pub coffee : String,
    pub size : String,
    pub qty : i32
}

//parameters needed for add_inventory
#[derive(Deserialize)]
pub struct AddInventoryParams{
    pub coffee: String,
    pub qty: i32
}

//checkout parameters
#[derive(Deserialize)]
pub struct CheckoutParams{
    pub cart_id : i32
}

/*
NOTE : had to use some outside resources for the handler setup, mainly i just looked online for basics, but i also 
had chat come up with a rough outline just so i had an idea of the layout of the handler setup
*/


//load_page: loads HTML of website and passes cart id to html
pub async fn load_page(State(state) : State<AppState>) -> Html<String>{

    //get number of orders from app state
    let num_orders = match state.num_orders.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            println!("num_orders mutex poisoned in load_page");
            //thread was poisoned but we can still continue
            poisoned.into_inner()
        }
    };
    let cart_id = *num_orders;

    //check if cart exists, if not insert new 'cart'/customerOrder
    //first must be mutable in order to add to it
    let mut carts = match state.carts.lock(){
        Ok(carts)=> carts,
        Err(_e)=>{
            println!("error loading customer carts hashmap");
            //will just show this on the page
            return Html("<h1>Something went wrong</h1>".to_string())
        }
    };

    //access the customerOrder in side the hashmap here
    let cart = carts
        .entry(cart_id)
        .or_insert_with(||CustomerOrder::new(cart_id as i32));


    //get inventory hashmap from appState
    let inventory = match state.inventory.lock() {
        Ok(inventory)=> inventory,
        Err(_e)=>{
            println!("Error getting inventory from Appstate");
            return Html("<h1>Inventory could not be initialized</h1>".to_string())
        }
    };

    //menu html, got help from ai to create the page elements the way i wanted
    let mut menu_html = String::new();

    //create the options to select coffee(s,m,l) from the coffees that are in stock
    for(coffee, stock) in &inventory.stock{

        let name = format!("{:?}", coffee);

        if *stock > 0{
            menu_html.push_str(&format!(
                r#"<h3><li>{name} (stock: {stock}) — 
                    <a href="/add?cart_id={cart_id}&coffee={name}&size=Small&qty=1">Small $5</a> | 
                    <a href="/add?cart_id={cart_id}&coffee={name}&size=Medium&qty=1">Medium $8</a> | 
                    <a href="/add?cart_id={cart_id}&coffee={name}&size=Large&qty=1">Large $12</a>
                </h3></li>"#
            ));

        } else {
            menu_html.push_str(&format!("<li>{name} — out of stock</li>"));
        }
    }
    //simple cart of items that have been added to cart, updates as things are added
    let mut cart_html = String::new();
    for item in &cart.items {
        cart_html.push_str(&format!(
            "<li>{:?} {:?} x{} — ${:.2}</li>",
            item.coffee, item.size, item.quantity, item.price
        ));
    }

    let html = format!(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Coffee Shop</title></head>
        <body>
            <h1>Coffee Shop</h1>
            <h3>
            <li><a href="/admin">Management Page</a></li>

            <h2>Menu</h2>
            <ul>{menu_html}</ul>

            <h2>Your Cart (id: {cart_id})</h2>
            <ul>{cart_html}</ul>
            <h3>Total: ${total:.2}</h3>
            <h3><a href="/checkout?cart_id={cart_id}">Checkout</a></h3>
        </body>
        </html>
    "#,
        menu_html = menu_html,
        cart_html = cart_html,
        cart_id   = cart_id,
        total     = cart.total_price,
    );

    Html(html)
}

// admin page: restock, view orders
pub async fn admin_page(State(state): State<AppState>) -> Html<String>{
    //get global inventory
    let inventory = match state.inventory.lock(){
        Ok(inv) => inv,
        Err(e) => return Html(format!("Inventory error: {e}").to_string())
    };

    //initialize empty html string to push onto
    let mut html = r#"
        <title>Management Page</title>
        
        <h1>Inventory</h1>
        "#.to_string();

    //iterate through coffees, find stock and name, give option to increase stock and show current stock
    for(coffee, stock) in &inventory.stock{

        let name = &coffee.to_str();

        html.push_str(&format!(
            r#"<h3><li> <a href="/inventory/add?coffee={name}&qty=10">+10 {name}</a> ({stock} in stock)
            </li></h3>"#
        ));
    }

    html.push_str(r#"<h2><a href="/">Home Page</a></h2>"#);
    Html(html)
}



//Query allows for deserialize to map our AddOrderParams struct with the values from the URLs
pub async fn add_item(State(state) : State<AppState>, Query(params) : Query<AddOrderParams>) -> Redirect{

    //collect deserialized parameters
    let coffee = parse_coffee(&params.coffee);
    let size = parse_size(&params.size);
    let amount = &params.qty;

    //believe using .lock() here is right because we dont want multiple threads editing
    //error checking
    let mut carts = match state.carts.lock(){
        Ok(carts) => carts,
        Err(e) =>{
            println!("error with cart initialization: {e}");
            return Redirect::to("/error");
        }
    };

    //error checking
    let mut inventory = match state.inventory.lock(){
        Ok(inventory) => inventory,
        Err(_e) =>{
            println!("error initializing inventory");
            return Redirect::to("/error")
        }
    };

    //finding the entry in the hashmap and if it doesnt exist we insert the a customer order with the current cart_id
    let cart = carts
        .entry(params.cart_id)
        .or_insert_with(|| CustomerOrder::new(params.cart_id as i32));

    let amt_reduce = Size::amount(&size);
    if inventory.reduce_stock(coffee, amt_reduce){
        cart.add_item(coffee, Roast::Medium, size , *amount);
    }  

    Redirect::to(&format!("/?cart_id={}", params.cart_id)) 
}


// this function allows us to restock coffee
pub async fn add_inventory(State(state) : State<AppState>, Query(params) : Query<AddInventoryParams>) -> Redirect{

    //parse coffee from params
    let coffee = parse_coffee(&params.coffee);

    //error checking: open inventory
    let mut inventory = match state.inventory.lock(){
        Ok(inventory)=> inventory,
        Err(_e)=>{
            println!("error getting inventory");
            return Redirect::to("/error")
        }
    };

    //call restocking  function
    inventory.add_stock(coffee, params.qty);

    //access db
    let db = match state.db.lock(){
        Ok(db) => db,
        Err(e) =>{
            println!("db connection was poisoned");
            e.into_inner()
        }
    };

    //update db stock
    match db.increase_stock(coffee){
        Ok(_) => (),
        Err(e) => println!("error increasing coffee stock: {e}")
    };

    //redirect back to admin
    Redirect::to("/admin")
}

/*
function checkout: takes in the state and checkout params, removes things from database that were in customer's cart,
adds checkout struct/db entry, etc.
*/
pub async fn checkout(State(state): State<AppState>, Query(params): Query<CheckoutParams>) -> Html<String>{
    //access cart struct based on id, get everything in the order
    let cart_id = &params.cart_id;

    //get carts, error check as well
    let carts = match state.carts.lock() {
        Ok(c) => c,
        Err(poisoned) => {
            println!("carts mutex poisoned");
            poisoned.into_inner()
        }
    };

    //find specific cart corresponding to id, and error check
    let cart = match carts.get(&cart_id) {
        Some(c) => c,
        None => {
            return Html("Cart not found".to_string());
        }
    };

    //initialize html string
    let mut items_html = String::new();

    //iterate through cart, pushing items onto the string to display.
    for item in &cart.items {
        items_html.push_str(&format!(
            "<h3><li>{:?} {:?} x{} - ${:.2}</li></h3>",
            item.coffee,
            item.size,
            item.quantity,
            item.price
        ));
    }

    //checkout page: display items and give customer option to confirm or go back
    let html = format!(
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Checkout</title>
    </head>
    <body>
        <h1>Order Confirmation</h1>
        <h2>Cart ID: {cart_id}</h2>

        <ul>
            {items_html}
        </ul>

        <h3><strong>Total: ${total:.2}</strong></h3>

        <h3><a href="/?cart_id={cart_id}">Back to Shop</a></h3>
        <h3><a href="/confirm_checkout?cart_id={cart_id}">Confirm Order</a></h3>
    </body>
    </html>
    "#,
    cart_id = cart_id,
    items_html = items_html,
    total = cart.total_price
    );

    //return html
    Html(html)
}

//confirm_checkout: confirms transaction, updates database
pub async fn confirm_checkout(State(state): State<AppState>, Query(params): Query<CheckoutParams>) -> Html<String> {

    //get params from Query
    let cart_id = params.cart_id;

    //get carts from state
    let mut carts = match state.carts.lock() {
        Ok(c) => c,
        Err(poisoned) => {
            println!("carts mutex poisoned");
            poisoned.into_inner()
        }
    }; 

    //find cart corresponding to id
    let cart = match carts.remove(&cart_id) {
        Some(c) => c,
        None => {
            return Html("Cart already checked out or missing".to_string());
        }
    };


    //html page for checkout confirmation
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Order Complete</title></head>
        <body>
            <h1>Thank you for your purchase!</h1>
            <h3>Your order #{cart_id} has been confirmed.</h3>
            <h3>Total Paid: ${total:.2}</h3>
            <a href="/">Start New Order</a>
        </body>
        </html>
        "#,
        cart_id = cart_id,
        total = cart.total_price
    );

    //get database from state variables
    let db = match state.db.lock(){
        Ok(db) => db,
        Err(e) =>{
            println!("db connection was poisoned");
            e.into_inner() //still gives database access
        }
    };

    //add order to database
    match db.insert_order(cart_id, cart.total_price){
        Ok(_) => (),
        Err(e) => println!("error adding order to database: {e}"),
    };

    //then we can add each item and amount from cart to the database as well
    for item in &cart.items {
        match db.insert_product_order(cart_id, item){
            Ok(_) => (),
            Err(e) => println!("failed to add product_order to database: {e}"),
        };
        //reduce database stock as well here
        match db.reduce_stock(item){
            Ok(_) => (),
            Err(e) => println!("failed to add product_order to database: {e}"),
        };
    }

    //increment global num_orders so we get a new cart_id 
    match state.num_orders.lock(){
        Ok(mut num_orders) => *num_orders += 1,
        Err(e) => {
            println!("error getting num orders in confirm_checkout {e}");
        }
    };

    Html(html)
}