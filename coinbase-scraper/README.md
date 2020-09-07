# Scrapper for the coinbase stock exchange.


It consists of multiple components:

### WebSocket client:

So the WebSocket client works like this:

- Wait till initial connection established.
  - Wait with manual backoff so that you can emit warning if connection was 
    not issued in 15 seconds, after client creation. 
  

- Connect (with rate limit of 1 connect every 500 ms)
    - On error try to reconnect but obeying the limit.  
    - Within 5 sec send Subscribe
      - On error drop socket and to reconnect.
    - Consume messages WS stream:
       - On Text msg:
          - Parse and Consume message
            - On parse error or error msg just log and ignore.
            - On a valid message do something (This can also fail).
       - On close message drop socket and reconnect.
       - On read error drop socket and reconnect.
       - On everything else just ignore.     
        
        
It should be possible to place new subscriptions from outside of the client (
    e.g. we have some other thread which periodically checks if there are any new
    products (markets) supported on the exchange, once detected Subscribe or Unsubscribe
    message is issued.  
). 


### Persistance

We should push every new message in some persistance module. Also if we detect some missing messages, 
or messages that are not in order, we should then go to rest API to get these data. This can only
be detected by the component that consumes the stream (but we can make the composite visitor and visitor decorator)   
