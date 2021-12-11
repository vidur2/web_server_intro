use std::net::{ TcpListener, TcpStream };
use std::io::{ Read };
use std::fs;
use std::io::Write;
use rayon;


const URL: &str = "127.0.0.1:7878";

struct Page {
    path: Option<String>,
    name: Option<String>,
}

fn main(){
    // Stores listeners
    let mut page_structure: Vec<Page> = Vec::new();
    page_structure = get_pages();

    let listener = TcpListener::bind(URL).unwrap();
    let pool  = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();
    
    for stream in listener.incoming(){
        let stream = stream.unwrap();
        pool.install(|| render_page(stream, &page_structure));
    }
}

fn render_page(mut stream: TcpStream, page_info: &Vec<Page>){
    println!("{}", rayon::current_thread_index().unwrap());
    
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let mut page_exists = false;
    let mut current_path = String::new();

    for page in page_info.iter(){
        if buffer.starts_with(page.name.as_ref().unwrap().as_bytes()){
            current_path = String::from(page.path.as_ref().unwrap().as_str());
            page_exists = true;
        }
    }

    if page_exists{
        let rendered_html = fs::read_to_string(current_path).expect("Invalid HTML");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            rendered_html.len(),
            rendered_html
        );

        println!("Connected!");
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

}

fn get_pages() -> Vec<Page>{
    let mut page_structure = Vec::<Page>::new();
    for entry in fs::read_dir("./pages").expect("Directory Does not exist"){
        let entry = entry.expect("No entry inside of the directory");
        let path = entry.path();
        let file_extension: &str = path.extension().unwrap().to_str().expect("Error casting file extension to string");
        if file_extension == "html"{
            let file_name: &str = path.file_stem().unwrap().to_str().expect("Error casting to string");
            if file_name == "index"{
                let current_page = Page {
                    path: Some(String::from(path.to_str().expect("Failed to cast path to string"))),
                    name: Some(String::from("GET / HTTP/1.1\r\n"))
                };
                page_structure.push(current_page);
            } else {
                let mut request_start = String::from("GET / HTTP/1.1\r\n");
                request_start.insert_str(5, file_name);
                let current_page = Page {
                    path: Some(String::from(path.to_str().expect("Failed to cast path to string"))),
                    name: Some(request_start)
                };
                page_structure.push(current_page);
            }
        };
    }

    page_structure
}