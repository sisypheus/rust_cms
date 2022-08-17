# CMS intended for blogs, made in rust with actix-web and sqlite

It's made to support only one account to be able to login, as only one administrator would need to publish new posts. The credentials are stored in the .env file.

The authentication is session-based with HTTP session cookies.

## Routes 
[GET] /posts => retrieve all posts stored in database.    
[GET] /posts/{id} => retrieve post from id.  
🔒[POST] /posts/save => save post in database.  
[POST] /login => login with saved credentials.  

## Run the application

#### Requirements 
* Setup db :   
```cd db```
```bash setup_db.sh```

* Have rust installed, you can install it from [here](https://www.rust-lang.org/tools/install)
* In a terminal : ```cargo run```

That's it, the application will be available on your local machine, on port 8080.
