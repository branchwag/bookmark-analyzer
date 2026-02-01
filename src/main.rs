mod browser;
mod ollama;
mod server;

#[tokio::main]
async fn main() {
    println!("🔍 Detecting browser and reading bookmarks...\n");

    let bookmarks = match browser::get_bookmarks() {
        Ok(bm) => {
            println!("✅ Found {} bookmarks\n", bm.len());
            bm
        }
        Err(e) => {
            eprintln!("❌ Error reading bookmarks: {}", e);
            return;
        }
    };

    println!("🤖 Analyzing bookmarks with Ollama...");
    println!("   (This may take a moment)\n");

    let analysis = match ollama::analyze_bookmarks(&bookmarks).await {
        Ok(result) => {
            println!("✅ Analysis complete!\n");
            result
        }
        Err(e) => {
            eprintln!("❌ Error analyzing bookmarks: {}", e);
            eprintln!("\n💡 Make sure Ollama is running:");
            eprintln!("   docker-compose up -d");
            eprintln!("   docker exec -it bookmark-analyzer-ollama-1 ollama pull llama3.2");
            return;
        }
    };

    if let Err(e) = server::serve(analysis, bookmarks.len()).await {
        eprintln!("Server error: {}", e);
    }
}
