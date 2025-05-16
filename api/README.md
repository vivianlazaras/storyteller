# StoryTeller API
Why make a seperate API in Go?
Because I'm lazy and I don't like the status of current rust ORMs they are unweildy and difficult to setu, GORM + easy HTTP and OIDC integration make deploying the API and database backend much easier.

The Advantage:
This also provides the distinct advantage that if the web app crashes the API won't, it also allows for protocol specification for alternative web interfaces to implement the protocol.

