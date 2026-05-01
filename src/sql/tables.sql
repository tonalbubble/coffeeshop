drop table if exists product;
drop table if exists orders;
drop table if exists product_order;

CREATE TABLE IF NOT EXISTS product(
    product_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    amount INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS orders(
    order_id INTEGER PRIMARY KEY AUTOINCREMENT,
    total_price REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS product_order(
    product_id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL,
    coffee TEXT NOT NULL,
    roast TEXT NOT NULL,
    size TEXT NOT NULL,
    quantity REAL NOT NULL,
    price REAL NOT NULL
);