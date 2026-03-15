
# jamcounter

Rust crate which uses a large-language model of your choosing to parse
GMC Jam votes and ranks and outputs the final scoring.

Supports both local and cloud LLMs.

To run with OpenAI, set the `JAM_OPENAI_API_KEY` environment variable
and execute `cargo run` (set `voting_post_url` in `config.toml` to
adjust the URL to pull from). For a local LLM, set `JAM_OPENAI_URL`
and `JAM_LLM_MODEL` as appropriate instead.

## Disclaimer

I am not officially affiliated with the GMC Jam. This project is just for fun :)
