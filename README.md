Http types for the fire http crate.

At the moment these types are more suitable
for server implementations than for clients.

## Features

### hyper_body
Adds support for the `hyper::Body` type in `Body`.

### json
Adds json serialization and deserialization support for
the `Body` type and in combination with the feature `encdec`
also to the `HeaderValues`.

### timeout
Adds the `BodyTimeout` type, allowing to set a timeout
for reading from the body.

### encdec
Adds percent encoding and decoding support for the
`HeaderValues` type.