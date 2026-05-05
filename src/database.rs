use rusqlite::{Connection, Error};
use std::fs;
use std::collections::HashMap;
use crate::model::{ItemOrder, Coffee, Size};
use crate::parse::parse_coffee;

//database struct: name is .db file name, conn is connection
#[derive(Debug)]
pub struct Database{
    pub name: String,
    pub conn: Connection
}

impl Database{
    //new create a database if we can successfully connect, otherwise propagate the error
    pub fn new(name: String) -> Result<Database, Error>{
        match Connection::open(&name){
            Ok(c) => Ok(Database{
                name:name,
                conn:c
            }),
            Err(e) => {
                Err(e)
            }
        }
    }

    //read tables.sql file, then execute batch of statements
    pub fn create_tables(&self) -> Result<(),Error>{
        let sql = fs::read_to_string("sql/tables.sql").expect("failed to read tables.sql");
        self.conn.execute_batch(&sql)?;
        Ok(())
    }

    //inserts coffee names and inventory from products.sql into tables
    pub fn insert_products(&self) -> Result<(),Error>{
        let sql = fs::read_to_string("sql/products.sql").expect("failed to read products.sql");
        self.conn.execute_batch(&sql)?;
        Ok(())
    }

    //dynamically insert a new order into orders table
    pub fn insert_order(&self, order_id : i32, total_price : f32) -> Result<(), Error>{
        self.conn.execute(
            "INSERT INTO orders (order_id, total_price)
             VALUES (?1, ?2)",
            (order_id, total_price),
        )?;
        Ok(())
    }

    //dynamically add a product_order to the db, reducing stock as well.
    pub fn insert_product_order(&self, order_id : i32, item : &ItemOrder ) -> Result<(), Error>{
        self.conn.execute(
            "INSERT INTO product_order 
            (order_id, coffee, roast, size, quantity, price)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                order_id,
                item.coffee.to_str(),
                item.roast.to_str(),
                item.size.to_str(),
                item.quantity,
                item.price,
            ),
        )?;

        Ok(())
    }

    //reduce stock in db after customer checks out
    pub fn reduce_stock(&self, order: &ItemOrder) -> Result<(), Error> {
        let coffee = Coffee::to_str(&order.coffee);
        let qty = order.quantity*Size::amount(&order.size);

        self.conn.execute(
            "UPDATE product
            SET amount = amount - ?1
            WHERE name = ?2",
            (qty, coffee),
        )?;
        Ok(())
    }

    //increase stock when restocking happens
    pub fn increase_stock(&self, coffee:Coffee) -> Result<(), Error> {
        let coffee = Coffee::to_str(&coffee);

        self.conn.execute(
            "UPDATE product
            SET amount = amount + 10
            WHERE name = ?1",
            (coffee,),
        )?;
        Ok(())
    }

    //function to know what cart id to start on upon startup
    pub fn get_num_orders(&self) -> i32 {
        //use a single-value select statement to get the 
        let max_id; 
        match self.conn.query_row(
            "SELECT COALESCE(MAX(order_id), 0) FROM orders",
            [],
            |row| row.get(0),
        ){
            Ok(val) => max_id = val,
            Err(_) => {
                max_id = 1;
                println!("error loading number of orders");
            }
        };

        max_id
    }


}


//this function lets us connect to an existing database OR creaet a new one
//takes in bool on whether we want to start fresh or connect to an existing file
pub fn db_init(new_db: bool) -> Result<(Database,HashMap<Coffee,i32>), rusqlite::Error>{
    let mut stock = HashMap::new();
    let db = match Database::new("sql/coffee.db".to_string()){
        Ok(db) => {
            println!("Successfully connected to {}",{&db.name});
            match load_inventory(&db){
                Ok(inv) => stock = inv,
                Err(_) => println!("failed to read inventory from database")
            };
            db
        },
        Err(e) => {
            println!("Error connecting to database: {e}");
            return Err(e);
        }
    };

    //if it's a new database, we create tables and insert products
    if new_db{
        match db.create_tables(){
            Ok(_) => println!("Tables created successfully"),
            Err(e) => println!("Error creating tables: {e}")
        };

        match db.insert_products(){
            Ok(_) => println!("Products entered successfully"),
            Err(e) => println!("Error inserting products: {e}")
        }
    }
    Ok((db, stock))
}


//function to populate inventory upon startup
pub fn load_inventory(db: &Database) -> Result<HashMap<Coffee, i32>, Error>{
    let mut stock = HashMap::new();

    let mut statement = db.conn.prepare("SELECT name, amount FROM product")?;

    let rows = statement.query_map([],|row|{
        let coffee: String = row.get(0)?;
        let amount: i32 =  row.get(1)?;
        Ok((coffee,amount))
    })?;

    for row in rows{
        let (coffee,amount) = row?;
        let coffee = parse_coffee(&coffee);
        stock.insert(coffee, amount);
    }
    Ok(stock)
}