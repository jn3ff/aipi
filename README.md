## AiPI

I got tired of rewriting serde logic for different models in my projects; this crate provides a standardized interface & typeset for interacting with LLMs.

This is strongly a WIP with no timeline guarantees for future development.

## How to use:

* See src/environment.rs; set your api key with the key of the provider

* See examples/conversation.rs; use LLM client with a built config to send messages to your configured model (minimally spanning support at this time)

### Current feature state:

* Anthropic Sonnet 4 and OpenAI Gpt 5 are supported through LLM client with ModelConfig

### Short term roadmap:

* Implement serde plumbing for more providers (Gemini)
* Map more models within implemented providers
* Add intuitive model switching mid-chat
* Set up AI-AI plumbing
* Add support for more params
* Map environment variable keys to a registry to allow user flexibility; right now they are expected as static consts

### Medium term roadmap:

* Support local models
* Improve extensibility of type system such that users can provide their own definition composing the guts of ModelConfig
* Clean up crate for public consumption
* include support for MCP & tool calling
* include support for non-text models (image, video) within the same standardized client flow & messaging including non-text content

### Long term roadmap:

* Would like to get meta with MCP; "MCP in a box"; such that a set of meta-tools are provided to the model. This work might come as part of a sibling crate.
