# routecheck

A command line tool for figuring out which pattern in a route table matches
a given HTTP method and path.

Most web frameworks, API gateways, and reverse proxies pick the first route
in a list whose pattern matches the request. When that list grows past a
couple dozen entries, it stops being obvious by inspection which pattern a
given request will actually hit, especially once wildcards and multiple
similarly-shaped patterns are involved. `routecheck` answers that question
directly against a plain text route file, without needing to boot the
service or send a real request.

## Route file format

One route per line: a method, whitespace, then a pattern. `*` as the method
matches any verb. Blank lines and lines starting with `#` are ignored.

```
# routes.txt
GET  /
GET  /users
GET  /users/:id
GET  /users/:id/posts/:post_id
POST /users
*    /health
GET  /static/*rest
```

Patterns are made of static segments, `:name` parameters that capture a
single path segment, and a trailing `*name` wildcard that captures
everything left in the path.

## Usage

```
routecheck <routes-file> <method> <path>
```

Example, using the `routes.txt` above:

```
$ routecheck routes.txt GET /users/42/posts/7
match: GET /users/:id/posts/:post_id (routes.txt:4)
  id = 42
  post_id = 7

$ routecheck routes.txt GET /static/css/site.css
match: GET /static/*rest (routes.txt:7)
  rest = css/site.css

$ routecheck routes.txt DELETE /users/42
no match for DELETE /users/42
```

Routes are checked top to bottom and the first one that matches both the
method and the path wins, mirroring how most routers behave. Exit status is
0 on a match and 1 otherwise, so it can be used in scripts.

## Building

Standard library only, no external dependencies.

```
cargo build --release
```
