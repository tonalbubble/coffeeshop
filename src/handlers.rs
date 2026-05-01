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
//use serde::Deserialize;
use rand::random;

//use crate::Size;
use crate::parse::{parse_coffee, parse_size};

use serde::Deserialize;
use crate::AppState;
use crate::model::{CustomerOrder, Roast, Coffee, Size};
//use crate::parse::{parse_coffee, parse_size};



/*
these struct will allow us to handle the parameters we want by passingh them when we use a handler
this will be used for example when using the basic get/

will be passing these and the Appstate so we have access to cart, inventory and db
*/
#[derive(Deserialize)]
pub struct PageParameters{
    pub cart_id : Option<i32>
}


#[derive(Deserialize)]
pub struct AddOrderParams{
    pub cart_id : i32,
    pub coffee : String,
    pub size : String,
    pub qty : i32
}

#[derive(Deserialize)]
pub struct AddInventoryParams{
    pub cart_id : Option<i32>,
    pub coffee: String,
    pub qty: i32
}

#[derive(Deserialize)]
pub struct CheckoutParams{
    pub cart_id : i32
}

/*
gonna try to build a handler for when the user clicks a add item to order
before doing all the setup of the actual page

NOTE : had to use some outside resources for the handler setup, mainly i just looked online for basics, but i also 
had chat come up with a rough outline just so i had an idea of the layout of the handler setup
*/



pub async fn loadPage(State(state) : State<AppState>, Query(params) : Query<PageParameters>) -> Html<String>{

    // #TODO change this to not random!
    let cart_id = params.cart_id.unwrap_or_else(|| random::<i32>());


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
                r#"<li>{name} (stock: {stock}) — 
                    <a href="/add?cart_id={cart_id}&coffee={name}&size=Small&qty=1">Small $5</a> | 
                    <a href="/add?cart_id={cart_id}&coffee={name}&size=Medium&qty=1">Medium $8</a> | 
                    <a href="/add?cart_id={cart_id}&coffee={name}&size=Large&qty=1">Large $12</a>
                </li>"#
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

            <h2>Menu</h2>
            <ul>{menu_html}</ul>

            <h2>Restock</h2>
            <ul>
                <li><a href="/inventory/add?coffee=Arabica&qty=10&cart_id={cart_id}">+10 Arabica</a></li>
                <li><a href="/inventory/add?coffee=Columbian&qty=10&cart_id={cart_id}">+10 Columbian</a></li>
                <li><a href="/inventory/add?coffee=Robusta&qty=10&cart_id={cart_id}">+10 Robusta</a></li>
                <li><a href="/inventory/add?coffee=Excelsa&qty=10&cart_id={cart_id}">+10 Excelsa</a></li>
                <li><a href="/inventory/add?coffee=BreakfastBlend&qty=10&cart_id={cart_id}">+10 Breakfast Blend</a></li>
                <li><a href="/inventory/add?coffee=MidnightRoast&qty=10&cart_id={cart_id}">+10 Midnight Roast</a></li>
            </ul>

            <h2>Your Cart (id: {cart_id})</h2>
            <ul>{cart_html}</ul>
            <p>Total: ${total:.2}</p>
            <a href="/checkout?cart_id={cart_id}">Checkout</a>
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


//Query allows for deserialize to map our AddOrderParams struct with the values from the URLs
pub async fn addItem(State(state) : State<AppState>, Query(params) : Query<AddOrderParams>) -> Redirect{

    let coffee = parse_coffee(&params.coffee);
    let size = parse_size(&params.size);

    let amount = &params.qty;

    //believe using .lock() here is right because we dont want multiple threads editing
    //error checking
    let mut carts = match state.carts.lock(){
        Ok(carts) => carts,
        Err(e) =>{
            println!("error with cart initialization");
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


    if inventory.reduce_stock(coffee, *amount){
        cart.add_item(coffee, Roast::Medium, size , *amount);
    }  

    Redirect::to(&format!("/?cart_id={}", params.cart_id)) 
}

/*
function addInventory:

*/
pub async fn addInventory(State(state) : State<AppState>, Query(params) : Query<AddInventoryParams>) -> Redirect{

    let coffee = parse_coffee(&params.coffee);

    let mut inventory = match state.inventory.lock(){
        Ok(inventory)=> inventory,
        Err(_e)=>{
            println!("error getting inventory");
            return Redirect::to("/error")
        }
    };

    inventory.add_stock(coffee, params.qty);

    match params.cart_id {
        Some(_id)=> Redirect::to(&format!("/?cart_id={}", _id)),
        None => Redirect::to("/"),
    }
}

/*
function checkout: takes in the state and checkout params, removes things from database that were in customer's cart,
adds checkout struct/db entry, etc.
*/
pub async fn checkout(State(state): State<AppState>, Query(params): Query<CheckoutParams>) -> Html<String>{
    //access cart struct based on id, get everything in the order
    let cart_id = &params.cart_id;
    let carts = state.carts.lock().unwrap();
    let cart = carts.get(&cart_id).unwrap();

    //debug print
    println!("you clicked checkout. your cart id is {cart_id}");

    //initialize html string
    let mut items_html = String::new();

    //iterate through cart, pushing items onto the string to display.
    for item in &cart.items {
        items_html.push_str(&format!(
            "<li>{:?} {:?} x{} - ${:.2}</li>",
            item.coffee,
            item.size,
            item.quantity,
            item.price
        ));
    }

    //checkout page: display order confirmation and allow user to navigate back to home
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

        <p><strong>Total: ${total:.2}</strong></p>

        <a href="/?cart_id={cart_id}">Back to Shop</a>
        <a href="/confirm_checkout?cart_id={cart_id}">Confirm Order</a>
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

//confirm_checkout function: when called, 
pub async fn confirm_checkout(State(state): State<AppState>, Query(params): Query<CheckoutParams>) -> Html<String> {
    let cart_id = params.cart_id;

    let mut carts = state.carts.lock().unwrap();

    let cart = carts.remove(&cart_id).unwrap();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Order Complete</title></head>
        <body>
            <h1>Thank you for your purchase!</h1>
            <p>Your order #{cart_id} has been confirmed.</p>
            <p>Total Paid: ${total:.2}</p>
            <a href="/">Start New Order</a>
        </body>
        </html>
        "#,
        cart_id = cart_id,
        total = cart.total_price
    );
    Html(html)
}