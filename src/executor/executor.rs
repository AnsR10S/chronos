use crate::lexer::lexer;
use crate::executor::{pipeline, single};

pub fn execute_pipeline(input: &str) -> bool {
    let tokens = lexer::tokenize(input);
    let mut pipe_chunks: Vec<Vec<String>> = Vec::new();
    let mut current_chunk = Vec::new();

    // Split the tokenized input into chunks separated by the pipe operator
    for token in tokens {
        if token == "|" {
            pipe_chunks.push(current_chunk);
            current_chunk = Vec::new();
        } else {
            current_chunk.push(token);
        }
    }
    pipe_chunks.push(current_chunk);

    // Route to the appropriate executor based on chunk count
    if pipe_chunks.len() > 1 {
        pipeline::execute(pipe_chunks)
    } else {
        single::execute(pipe_chunks.remove(0))
    }
}
