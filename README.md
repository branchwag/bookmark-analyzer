# Bookmark Analyzer

Ok this might be a bit of vanity bait, but this app scans your bookmarks and gives you a synoposis of judgement.
Due to the sensitive nature of bookmarks (I really don't want to know what you are into), I am not hosting it, but rather offering the code so YOU can clone it down and get your own results on localhost. If you want to share your collection publicly, go for it on social media (I'm not your mom).

To run:
```
docker-compose up -d
docker exec -it bookmark-analyzer-ollama-1 ollama pull llama3.2
cargo run
```
