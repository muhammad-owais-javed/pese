//****Libraries Declaration****//
use rand::Rng;
use sha2::{Digest, Sha512};
use std::io::{Write, BufReader, BufRead};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

//Lightweigh http client for basic http client features
use mini_http;
use mini_http::{Request, Server};

//For Lazy Evaluation Statics: create Static in runtime
use lazy_static::lazy_static;

//For Providing Hashing Algorithm
use std::collections::HashMap;

//For Protecting Shared Data: Helpful in blocking threads
//and wait for the mutex locks to become available
use std::sync::Mutex;

// Application keeps many large HTML pages in RAM memory.
// Reduce the memory usage by compressing the text HTML pages.
use compressed_string::ComprString;

//*****************************//


// Search point address and port for non-wasm 'cargo run'
const ADDRESS: &str = "46.19.38.63"; // IP Address declaration Ahmia.fi
const DELAY: u64 = 10; // Delay time in seconds

// Alternative port for Ahmia search, 31000
lazy_static! {
    static ref PORT: Mutex<i32> = {
        Mutex::new(31000)
    };
}

//Struct to hold TCPStream value in connection
struct Stream { connection: TcpStream }

impl Stream {
  fn new(connection: TcpStream) -> Self {  //new connection establishment
      Stream { connection: connection }
  }
  fn restart(&mut self) {  //restarting connection
      self.connection = get_tcpstream().unwrap();
  }
}

lazy_static! {
    static ref STREAM: Mutex<Stream> = {
        Mutex::new(Stream::new(get_tcpstream().unwrap()))
    };
}

//Struct for Webitem account to hold fields like timestamp and data
struct Webitem { timestamp: u64, data: ComprString }

impl Webitem { // Save the compressed version of the string
  fn new(timestamp: u64, data: String) -> Self {
      Webitem { timestamp: timestamp, data: ComprString::new(&data) }
  }
  pub fn get_timestamp(&self) -> u64 { //For getting timestamp
      self.timestamp
  }
  pub fn get_data(&self) -> String {  //For getting data
      self.data.to_string()
  }
  pub fn get_size(&self) -> usize {  //for getting size of data
      self.data.compressed_len()
  }
}

lazy_static! {
    static ref QUERYMAP: Mutex<HashMap<String, Webitem>> = {
        Mutex::new(HashMap::new())
    };
}

lazy_static! {
    static ref RESULTMAP: Mutex<HashMap<String, Webitem>> = {
        Mutex::new(HashMap::new())
    };
}

//For getting Randomnumber range in between -2147483648 to 2147483647
fn randomnumber(min: i32, max: i32) -> i32 {  
    assert!(min < max);
    let number: i32 = rand::thread_rng().gen_range(min..max);
    assert!(min <= number && number < max);
    return number;
}

// A random number from -2147483648 to 2147483647 - 1, around 2^32 possibilities
lazy_static! {
    static ref RANDOMNUMBER: Mutex<i32> = {
        Mutex::new(randomnumber(i32::MIN, i32::MAX))
    };
}

fn timenow() -> u64 {
    // Current time: return the total number of seconds
    let seconds: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)  //Unix timestamp elapsed from 1 Jan 1970
        .unwrap()
        .as_secs();
    return seconds;
}

fn random_choice() -> (String, String) {
    // Selection of Random choice from the hashmap and return item.
    let size: i32 = QUERYMAP.lock().unwrap().len().try_into().unwrap();
    assert!(size > 0); // Size is always larger than 0

    // Random selection from the hashmap
    let selection: i32 = randomnumber(0, size); // 0 to size-1
    assert!(selection < size);

    let mut counter: i32 = 0;
    loop { // Return the selected item
        for (key, value) in QUERYMAP.lock().unwrap().iter() {
            if selection == counter {
                let copy_key = format!("{}", key);
                let copy_value = format!("{}", value.get_data());
                return (copy_key, copy_value); // Return values
            }
            counter += 1;
            assert!(counter < size);
        }
    }
}

fn suffle_select() -> Result<(String, String), (String, String)> {
    // Select of random item and remove it from the hashmap
    let size = QUERYMAP.lock().unwrap().len();
    
    if size == 0 { // No items so nothing to return
        return Err(("".to_string(), "".to_string())); // None 
    }

    let (copy_key, copy_value) = random_choice();
    QUERYMAP.lock().unwrap().remove(&copy_key);
    return Ok((copy_key, copy_value));
}

//For generating SHA512 hash of 64-bytes
fn hash512(data: &[u8]) -> String { 
    let mut hasher = Sha512::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

//For generating id from the SHA512 hash (initial 20 bytes)
fn id_hash(input: String) -> String {
    let number = RANDOMNUMBER.lock().unwrap();
    let str_hash = format!("/result/{}", hash512(format!("{} - {:?}", input, number).as_bytes()));
    return str_hash[0..20].to_string();
}

//Function for Redirecting HTML page to the hash URL
fn redirect_html(hash: String) -> String {
    // Return the redirect HTML page for the hash URL
    let html_string = format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
      "<!DOCTYPE html>",
      "<html>",
      "<head>",
      "<meta http-equiv='refresh' content='",
      DELAY,
      "; URL=",
      hash,
      "' />",
      "</head>",
      "<body>",
      "<p> Wait seconds: ",
      DELAY,
      "</p> <p>You'll be sent directly to the results page.</p>",
      "</body>",
      "</html>",
    );
    return html_string.to_string();
}

#[cfg(target_os = "wasi")]
fn get_server() -> Server {
    mini_http::Server::preopened().unwrap()
}

#[cfg(not(target_os = "wasi"))]
fn get_server() -> Server {
    mini_http::Server::new("127.0.0.1:34455").unwrap()
}

// When the target of the build is wasi
// Making TCP connection with the address 
#[cfg(target_os = "wasi")]
fn get_tcpstream() -> std::result::Result<TcpStream, Box<dyn std::error::Error>> {
    // Use existing TCP connections
    // Enarx has already established it (ahmif.fi:31000...31009)
    println!("Connect to {}:{}", ADDRESS, PORT.lock().unwrap());
    // NOTE: This is for wasm: if the original connection to the port 31000 is lost
    let desc = *PORT.lock().unwrap() - 31000 + 4; // The first call returns 4
    use std::os::wasi::io::FromRawFd;
    let stdstream = unsafe { std::net::TcpStream::from_raw_fd(desc) };
    *PORT.lock().unwrap() += 1; // NOTE: Increase the port for the next call
    Ok(stdstream)
}

// When the target of the build is not wasi
// Making TCP connection with the Address 
#[cfg(not(target_os = "wasi"))]
fn get_tcpstream() -> std::result::Result<TcpStream, std::io::Error> {
    println!("Connect to {}:{}", ADDRESS, PORT.lock().unwrap());
    let searchaddress = format!("{}:{}", ADDRESS, PORT.lock().unwrap());
    std::net::TcpStream::connect(searchaddress)
}

//Function to read content length of http packet
fn read_content_length(http: &String) -> i32 {
    let str_lines: Vec<&str> = http.lines().collect();
    for line in str_lines{
        if line.contains("Content-Length: ") {
            let str_parts: Vec<&str> = line.split_whitespace().collect();
            return str_parts[1].parse::<i32>().unwrap();
        }
    }
    return 0;
}

fn htmlpage(page: String) -> String {
    // Remove redirect search result links to point directly to the result pages
    let part1 = "<a\n                href=\"/search/search/redirect?search_term=";
    let replacement1 = format!("{}{}", "<!--", part1);
    let part2 = "&redirect_url=";
    let replacement2 = format!("{}{}", part2, "-->\n                <a href=\"");
    let mut htmlpage = page.replace(part1, &replacement1).replace(part2, &replacement2);
    // Add padding: Each HTML page is around 2 000 000 bytes = 2MB.
    let mut hasher = Sha512::new();
    hasher.update(randomnumber(0, 1000000000).to_string()); // Hash a random number
    let hash_str = format!("{:x}", hasher.finalize()); // Format the hash to string
    // 128 multiplied 1000..2000 times
    let pad = hash_str.repeat(randomnumber(1000, 2000).try_into().unwrap());
    // Size is 1 700 000 + variance
    while htmlpage.len() < 1700000 { // Add random nonsense and noise to the final size
        let padding = format!("{}{}{}", "\n    <!-- ", pad, " -->    \n</html>");
        htmlpage = htmlpage.replace("</html>", &padding); // Add at least 128 000 bytes
    }
    return htmlpage;
}

fn request(key: &String, value: &String) -> std::result::Result<String, String> {
    println!("Try to forward: {}", value);
    let mut stream = &STREAM.lock().unwrap().connection;
    let result_write = stream.write_all(value.as_bytes());
    match result_write {
        Ok(_)=> { println!("Forwarded: {}", value); },
        Err(_)=> { return Err("Failed request".to_string()); }
    }
    stream.flush().unwrap(); // Flush or server never responses
    // Receive data from TCP
    let mut reader = BufReader::new(stream);
    let mut result = String::new();
    let mut readbytes: i32 = 0;
    loop { // Read bytes until no data from the server
        let bytes = reader.read_line(&mut result).unwrap();
        if bytes == 0 { break; }
        if readbytes == 0 && result.contains("\r\n\r\n") {
            readbytes = read_content_length(&result);
            result = String::new();
        }
	if readbytes > 0 && result.len() >= readbytes.try_into().unwrap() { break };
    }
    if result.len() > 0 {
        println!("Received {:?} bytes", result.len());
        let page = htmlpage(result.to_string());
        println!("With added padding {:?} bytes.", page.len());
        RESULTMAP.lock().unwrap().insert(key.to_string(), Webitem::new(timenow(), page));
        let size = RESULTMAP.lock().unwrap().get(&key.to_string()).unwrap().get_size();
        println!("Compressed version consumes {} bytes RAM.", size);
        return Ok("Results OK".to_string()); // Done, ready, return
    }
    else { return Err(format!("Failed: {}", value).to_string()); }
}

//Function for Fetching Results
fn get_results(url: String) -> std::result::Result<Vec<u8>, String> {
    if !QUERYMAP.lock().unwrap().contains_key(&url) && !RESULTMAP.lock().unwrap().contains_key(&url) {
        return Err("Invalid URL".to_string());
    }
    // If results already available return the result
    if RESULTMAP.lock().unwrap().contains_key(&url) {
        return Ok(RESULTMAP.lock().unwrap().get(&url).unwrap().get_data().as_bytes().to_vec());
    }
    // Check that the user waited the delay
    if QUERYMAP.lock().unwrap().contains_key(&url) {
        let seconds = QUERYMAP.lock().unwrap().get(&url).unwrap().get_timestamp();
        if timenow() < (seconds + DELAY) {
            return Ok(redirect_html(url).as_bytes().to_vec()); // Wait till delay time is full
        }
    }
    // Else execute all queries
    while QUERYMAP.lock().unwrap().len() > 0 { // Process all the items in random order
        match suffle_select() { // Random selection which removes the item from the hashmap
            Ok((key, value)) => {
                for _i in 0..3 {
                    match request(&key, &value) {
                        Ok(_) => { break; },
                        Err(msg) => { println!("{}", msg);  }
                    }
                    STREAM.lock().unwrap().restart(); // Try with a new connection
                }
            },
            Err(_) => { println!("Error: failed to select random item"); }
        }
    }
    if RESULTMAP.lock().unwrap().contains_key(&url){
        return Ok(RESULTMAP.lock().unwrap().get(&url).unwrap().get_data().as_bytes().to_vec());
    }
    return Err("ERROR: no result".to_string());
}

//Function for the exection of user search query
fn execute_query(req: &Request) -> Vec<u8> {
    if !req.uri().to_string().contains("/result/") {
        return "Not Found".as_bytes().to_vec();
    }
    let mut bytes_list : Vec<u8> = Vec::new();
    let result = get_results(req.uri().to_string());
    match result {
        Ok(bytes)=>{
            bytes_list.extend(bytes);
        },
        Err(msg)=>{
            println!("Error: {}", msg);
        }
    }
    return bytes_list;
}

//Function for receiving user search query
fn collect_query(req: &Request) -> Vec<u8> {
    println!("Received: {}", req.uri());
    let httpget = format!("{}{}{}{}{}{}",
        "GET ",
        req.uri().to_string(),
        " HTTP/1.1\r\n",
        "Host: ",
        ADDRESS,
        "\r\n\r\n",
    );
    let hash = id_hash(req.uri().to_string());
    if !QUERYMAP.lock().unwrap().contains_key(&hash) {
        if !RESULTMAP.lock().unwrap().contains_key(&hash) {
            QUERYMAP.lock().unwrap().insert(hash, Webitem::new(timenow(), httpget.to_string()));
        }
    }
    let hash = id_hash(req.uri().to_string());
    let html_string = redirect_html(hash);
    return html_string.as_bytes().to_vec(); 
}

fn checksum(hash: &str, bytes: &[u8]) -> bool {
    // Check that hash matches to the hashed bytes
    hash == &hash512(bytes)[0..hash.len()] // Match the begining
}

// We do not trust resources outside of the enclave.
// When opening files we check it is not changed.
fn style_css() -> &'static [u8] {
    // We do not trust the unsafe world!
    let bytes = include_bytes!("../css/styles.css");
    // Check and verify it is the resource we think it is.
    assert!(checksum("f2421cb5603eb88797c55d92e979", bytes));
    return bytes;
}

//*****For verifying Correct Ahmia.fi CSS files are loaded*****//

fn style_arrow() -> &'static [u8] {
    let bytes = include_bytes!("../css/ddarrow.png");
    assert!(checksum("3b3f21d6b5644d05e47d1904f39", bytes));
    return bytes;
}

fn style_ahmiafi() -> &'static [u8] {
    let bytes = include_bytes!("../css/ahmiafi_black.png");
    assert!(checksum("308ea08e8e75", bytes));
    return bytes;
}

fn style_metro() -> &'static [u8] {
    let bytes = include_bytes!("../css/metro.jpg");
    assert!(checksum("7653f69165835bde229b0f9c536b2", bytes));
    return bytes;
}

//*************************************************************//


//For verifying correct ahmia.fi index page is loaded
fn index_page() -> &'static [u8] {
    let bytes = include_bytes!("../html/index.html");
    assert!(checksum("a21ff31225bb419b49ec9f792f3d93ef", bytes));
    return bytes;
}

//Function to run server, load ahmia.fi and return the user search results
fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Run server and return query results
    get_server()
        .tcp_nodelay(true)
        .start(move |req| match req.uri().path() {
            "/search/" => mini_http::Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(collect_query(&req))
                .unwrap(),
            "/static/images/ddarrow.png" => mini_http::Response::builder()
                .status(200)
                .header("Content-Type", "image/png")
                .body(style_arrow().to_vec())
                .unwrap(),
            "/static/images/metro.jpg" => mini_http::Response::builder()
                .status(200)
                .header("Content-Type", "image/png")
                .body(style_metro().to_vec())
                .unwrap(),
            "/static/images/ahmiafi_black.png" => mini_http::Response::builder()
                .status(200)
                .header("Content-Type", "image/png")
                .body(style_ahmiafi().to_vec())
                .unwrap(),
            "/static/css/normalize.css" => mini_http::Response::builder()
                .status(200)
                .header("Content-Type", "text/css")
                .body(style_css().to_vec())
                .unwrap(),
            "/" => mini_http::Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(index_page().to_vec())
                .unwrap(),
            _ => mini_http::Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(execute_query(&req))
                .unwrap(),
        })?;
    Ok(())
}

//****Main Function****//

pub fn main() {
    println!("Privacy Extension for Search Engines (PESE)");
    println!("Searches results from Ahmia.fi: {}:{}", ADDRESS, PORT.lock().unwrap());
    println!("\nSearch Query Mixer:");
    println!("Open on WebBrowser: http://127.0.0.1:34455/");
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);
    }
}

//*********************//
