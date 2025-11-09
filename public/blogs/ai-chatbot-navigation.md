# AI Assisted Website Navigation

**Published:** November 2025

In this article, I would like to discuss the possibility of using AI-assisted-navigation on websites to prevent information overload.

## What does AI Assisted Website Navigation Look Like?

One of the large issues that I can see within large companies is that their websites are oftentimes filled with plenty of information that, no matter how insightful the UX Designer is, the user will still struggle to find the content that they need to see. 

Henceforth, I came up with the idea of AI to assist users in finding the right content they need. This further minimizes information overload and simplifies the process of finding the right content they want to see on a website.

## How is it built

It uses 3 major components: A Chatbot UI, an API to your LLM of choice (mine's Gemini), and a knowledge base. 

### Chatbot UI

This one's self-explanatory. You'd need a Chat-UI in order to interact with the LLM. The switching logic has to be done within the Chatbot UI as well. The difficulty that I encountered is to make sure that the sessions are saved all throughout the chats. What I've done so far is to save the chats within LocalStorage. While this is a "prototype" solution, this solves the problem that I've had. My recommendation is to set up session with the LLM so that you can save the messages on the backend side of things (I have yapped enough about a topic I don't know about. I'd love to know if what I'm saying is theoretically possible).

### API to your LLM of choice

This one's also self-explanatory but based on what I've researched, you can make do with extremely light models and you don't need much complexity as long as the proper context is provided.

### Knowledge Base

This one's slightly more complicated for me to implement. I have chosen to use a file for the implementation of a knowledge base so that I could simplify the process. However, this doesn't scale. The best practice here would most likely be to set up some RAG database or some implementation of a NoSQL Database where you can query the link information.

## Use Cases

For a use case, imagine you are a large organization that is selling various products. Instead of users having to scroll through lists of products to find the right one, you can have them use a chatbot to inquire about a specific product. You then redirect them to that product, so they can review and then buy.

Another use case is for large SaaS projects, where you don't have to read various documentation to find a specific feature you want to use. You can have AI land you on that feature instead by simply asking. 

## Potential Limitations
1. Scale
As you try to scale the chatbot further, the more tokens you would be consuming, as of right now, I am not sure how to implement this on a scale that is sustainable for SME's or Startups.

2. Complexity
As you scale the project further, the more the knowledge base becomes more and more complex. I have been able to use a single static file as my basis for the navigation knowledge bases. However, I acknowledge that the bigger the project, the more complex the need would be for a database to store your information. 

## Test & Source Code
If you guys like, feel free to try for yourselves.
Link: [https://willvincentparrone.com](https://willvincentparrone.com)
Github: [https://github.com/kyahwill/rust_portfolio](https://github.com/kyahwill/rust_portfolio)

