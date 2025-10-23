# Federation
Federation provides a means for homeservers to communicate with each other in controlled ways.

## The Problem
It's notoriously difficult within decentralized systems to encourage active participation, this is I think due in large part to how the internet (IP) is currently operating, more specifically how firewalls and network address translation (NAT) layers prevent easy bidirectional, secure communication.
There is also an extensive problem when it comes to content moderation. This is especially an issue for end to end encrypted self hostable systems.

## Potential Solutions
1. Authorized hosting servers only.
2. Text only content.
3. Delayed posting to the network until content has been verified safe by trusted moderators.
4. Hoster / Verifier Reputation systems. 

## Plans Moving Forward
Encryption and privacy are without a doubt a benefit, but anonymity doesn't support healthy communities over all.
My approach will be to take a harm reduction approach, separating core functionality based on the levels of harm it could potentially do,
granting rights to perform certain services only after peers have proven themselves trustworthy within the network.

Initially a peer will have the rights to post time delayed text only comments using their public/private key pair.
Then as trust grows and good behavior is modeled, they will gain rights to publish lengthier content, including their own works,
and their own interpretations of others works as a time delayed text post.

Once reputation is high enough, they will gain the right to post time delayed images, and non time delayed text posts.
With high enough reputation by existing moderators they will be offered moderation privileges themselves, which can be revoked.

The highest level of access will be that of an identity provider mapped to a user's public key, and signed by the identity provider.
This approach ensures privacy without anonymity, so that while to content in a community or conversation may not be known to everyone, who is in that conversation will be.

