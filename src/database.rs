use rusqlite::{Connection, Statement, Error};

use crate::model::ItemOrder;

//database struct: name is .db file name, conn is connection
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

    pub fn insert_order(&self, customer_id : i32, total_price : f32) -> Result<i64, Error>{
        self.conn.execute(
            "INSERT INTO orders (customer_id, total_price)
             VALUES (?1, ?2)",
            (customer_id, total_price),
        )?;

        Ok(self.conn.last_insert_rowid())
    }


    pub fn insert_order_item(&self, order_id : i64, item : &ItemOrder ) -> Result<(), Error>{
        self.conn.execute(
            "INSERT INTO order_items 
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

    //creates the tables that we will need
    pub fn create_tables(&self) -> Result<(),Error>{

        //jus the products we have on the menu
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS product(
                product_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                base_price DECIMAL NOT NULL
                )",())?; //error propagation


        //what we have in stock
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS inventory(
                product_id INTEGER PRIMARY KEY,
                stock INTEGER NOT NULL,
                FOREIGN KEY(product_id) REFERENCES product(product_id)
                )",())?;
        

        //CustomerOrder table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS orders(
                order_id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_id INTEGER,
                total_price REAL NOT NULL,
            )",())?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS order_items(
                item_id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                coffee TEXT NOT NULL,
                roast TEXT NOT NULL,
                size TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
            )",())?;
        Ok(())
    }

    //drop tables if we need to (not commonly needed)
    pub fn drop_tables(&self)-> Result<(),Error>{
        self.conn.execute(
            "DROP TABLE IF EXISTS product;",())?;
        Ok(())
    }
}
