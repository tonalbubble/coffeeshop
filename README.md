COFFEESHOP SIMULATION - CS310(RUST)

Multi-threaded webapp using AXUM for the web framework and SQlite for a database to 
simulate a coffeeshop with the ability to purchase coffee by the bag
and track inventory on the admin side. 

------------------------------------------------

Features:
user carts -> each user is assigned a cart-id that is carried through the use of the webpage via the URL
live inventory tracking -> as users add to their cart the inventory is tracked and updated accordingly
inventory restock -> basic restock function to add more coffee to the inventories, a little simpler than planned
but ideally this would end up as something that uses user authentication
current cart -> as users click the URLs to order their purchase will be displayed in a cart section of the page
checkout feature -> users can checkout and this clears the cart, storing the purchase in the database
mulitple-concurrent users -> Axum and Tokio were used as handlers for the multi-threaded functionality

--------------------------------------------------
Setup:
cargo run in terminal of VScode, then go to localhost:7008
--------------------------------------------------

Resources used:
Language : Rust
Axum -> web framework
Tokio -> Async multi thread
SQlite -> database
Serde -> serialization

---------------------------------------------------

Project Structure:

src/
    main.rs -> Appstate definition and webframework construction
    database.rs -> Database initialization and member functions
    model.rs -> all core structs/data types and implementation
    handlers.rs -> route handlers
    parse.rs -> helper functions to parse strings to enum types

----------------------------------------------------









