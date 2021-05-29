#HTTP-broker

Simple HTTP task broker

##API

`PUT /{QUEUE_NAME}` -- Put Message in a queue (message is a byte array from body)  
`GET /{QUEUE_NAME}` -- Retrieve Message from a queue. `?timeout` parameter to specify time to block for waiting for parameter. Returns 404 if queue is empty


##Example 
```
PUT /a
one
PUT /a
two
PUT /b
three
GET /a
>>>200 OK : one
GET /b
>>> 200 OK: three
GET /b
>>> 404 Not Found
GET /a
>>> 200 OK : two
GET /a
>>> 404 Not Found
```