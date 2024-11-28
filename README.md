# `openapi-gen`: Generation of Rust Types and Traits from an OpenAPI Spec

After some research, it turns out that the existing options for generating Rust servers from an OpenAPI spec are pretty bad.
The OpenAPI spec is broad enough, and the server ecosystem is wide enough, that there really isn't anything best-in-class right now.

We solve this by punting the can down the road: you, the user, get to write all your own server code, using whatever framework you like (or none at all!).
We don't mess with it; it's on you.

What this _does_ provide is:

- A library handling generation of Rust types and traits from an OpenAPI spec
- A CLI for interacting with that library (feature-gated on the `cli` feature)
- A build script library interface (in the `openapi-gen-build` crate)
- A macro (`include_openapi_defs`) for including the generated code

## ARCHIVE NOTICE

Archiving on 28 November 2024. I no longer have a practical need for this tool, and am uninterested in maintaining it. I recommend forking. 

## WARNING

This README is a declaration of intent; while this code is being actively developed, it is not yet ready to go, and features may be absent without explanation.
Treat everything you read here with a declaration of salt; read the code to know its state.

## Including this in your Rust project

Because this project depends on [`accept-header`](https://github.com/coriolinus/accept-header) which is not hosted on `crates.io`, this is also not published on `crates.io`. It is likely to confuse people if a simple `cargo install` or `cargo add` fails on a project there without additional configuration, so for courtesy, I am not publishing this there.

### Git Dependency

```toml
openapi-gen = { version = "0.1.0", git = "https://github.com/coriolinus/openapi-gen.git" }
```

### Cloudsmith Package Repository

[![Hosted By: Cloudsmith](https://img.shields.io/badge/OSS%20hosting%20by-cloudsmith-blue?logo=cloudsmith&style=for-the-badge)](https://cloudsmith.com)

Add the following lines to [a relevant `.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html):

```toml
[registries.finvia-accept-header]
index = "sparse+https://cargo.cloudsmith.io/finvia/accept-header/"

[registries.finvia-openapi-gen]
index = "sparse+https://cargo.cloudsmith.io/finvia/openapi-gen/"
```

Then depend on it as

```toml
openapi-gen = { version = "0.1.0", registry = "finvia-openapi-gen" }
```

Package repository hosting is graciously provided by  [Cloudsmith](https://cloudsmith.com).
Cloudsmith is the only fully hosted, cloud-native, universal package management solution, that
enables your organization to create, store and share packages in any format, to any place, with total
confidence.

## Crate Features

This crate defines a number of features, none of which are enabled by default. These enable downstream users to limit their dependencies according to what features they need.

| Feature | Notes |
| --- | --- |
| `cli` | This feature builds a command-line interface with which to drive this create. Unnecessary if using `openapi-gen-build` in a build script. |
| `scripts` | This feature builds several utilities which are mostly interesting only to developers of this crate. |
| `bytes` | Enables the `Bytes` well-known type, which encodes binary data as Base64. |
| `integer-restrictions` | This feature enables the `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`, and `multipleOf` restrictions on integers. |
| `string-pattern` | This feature enables the `pattern` restriction on strings. |

## Integrating this into your code

1. Add the `opeanpi-gen-build` package as a build dependency.

    ```bash
    cargo add --build openapi-gen-build
    ```

2. Add the `openapi-gen` package as a dependency.

    ```bash
    cargo add openapi-gen
    ```

3. Run the build step as part of your build script.

    ```rust
    fn main() -> Result<(), Box<dyn std::error::Error>> {
        openapi_gen_build::generate_openapi("openapi/service.openapi.yaml")?;
        Ok(())
    }
    ```

4. Embed the generated code into your crate.

    ```rust
    mod openapi {
        openapi_gen::include!("service");
    }
    ```

    Note that you define the containing module, so if you need to implement methods or traits on the generated types, you are free to do so.

## What gets generated

**This crate does not implement a server**.

This crate does not implement a server.

This crate generates _models_ and _traits_ as defined by an OpenAPI specification.

### Routes

Routes defined in the `#/paths` section of the document are collected into a `trait Api`. This trait can and should be implemented both by the client (whose implementation makes a network call to the server), and by the server (whose implementation responds to the appropriate call).

```yaml
paths:
  "/kudos":
    post:
      operationId: "postKudos"
      requestBody:
        description: request body for a freeform render request
        content:
          "application/json":
            schema:
              "$ref": "#/components/schemas/PostKudo"
        required: true
      responses:
        '201':
          description: "accepted kudo"
        default:
          description: an error occurred; see status code and problem object for more information
          content:
            "application/problem+json":
              schema:
                "$ref": "https://opensource.zalando.com/restful-api-guidelines/models/problem-1.0.1.yaml#/Problem"
```

#### Request Enum

In the case that the request permits more than a single type of content, if the schemas are not identical, then a request enum is created.

```yaml
operationId: multiRequests
requestBody:
    description: request body that can accept two content types
    required: true
    content:
        "application/json":
            schema:
                "$ref": "#/components/schemas/JsonType"
        "multipart/form-value":
            schema:
                "$ref": "$/components/schemas/FormType"
```

```rust
pub enum MultiRequestsRequest {
    ApplicationJson(ApplicationJson),
    MultipartFormValue(MultipartFormValue),
}
```

In the case that `required: true` is not set, the request type is wrapped in an `Option`.

```yaml
operationId: optionalRequestBody
requestBody:
    description: request body is optional
    # required: true
    content:
        "application/json":
            schema:
                "$ref": "#/components/schemas/JsonType"
```

```rust
pub type OptionalRequestBodyRequest = Option<JsonType>;
```

In the case that the request body is required and permits only a single type of content, the request type is unified.

```yaml
operationId: sameRequest
requestBody:
    description: request body that can accept two content types
    required: true
    content:
        "application/json":
            schema:
                "$ref": "#/components/schemas/ReqType"
        "multipart/form-value":
            schema:
                "$ref": "$/components/schemas/ReqType"
```

```rust
pub type SameRequestRequest = ReqType;
```

#### Response Enum

Every enumerated response variant is collected into a response enum by status code. Each variant contains an appropriate struct.

Streaming responses are not currently supported. In the future they may be supported by an extension annotation on the schema declaration.

```yaml
responses:
  '200':
    description: "rendered PDF document"
    content:
      "application/pdf":
        schema:
          type: string
          format: binary
  '400':
    description: "it was not possible to produce a PDF given this request"
    content:
      "application/problem+json":
        schema:
          "$ref": "#/components/schemas/RenderError"
  '503':
    description: "upstream service was not available; try again later"
    content:
      "application/problem+json":
        schema:
          "$ref": "https://opensource.zalando.com/restful-api-guidelines/models/problem-1.0.1.yaml#/Problem"
  default:
    description: an error occurred; see status code and problem object for more information
    content:
      "application/problem+json":
        schema:
          "$ref": "https://opensource.zalando.com/restful-api-guidelines/models/problem-1.0.1.yaml#/Problem"
```

```rust
type Ok_ = Vec<u8>;
type ServiceUnavailable = openapi_gen::reexport::http_api_problem::HttpApiProblem;
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
pub enum RenderPdfResponse {
    Ok(Ok_),
    BadRequest(RenderError),
    ServiceUnavailable(ServiceUnavailable),
    Default(Default_),
}
```

Given a schema, responses will always contain an appropriate object. Without that, responses will be set on a best-effort basis from the content type. If the `content` section is missing, that response will be a unit variant.

#### `trait Api`

```yaml
post:
  operationId: "postKudos"
  requestBody:
    description: request body for a freeform render request
    content:
      "application/json":
        schema:
          "$ref": "#/components/schemas/PostKudo"
    required: true
  responses:
    '201':
      description: "accepted kudo"
    default:
      description: an error occurred; see status code and problem object for more information
      content:
        "application/problem+json":
          schema:
            "$ref": "https://opensource.zalando.com/restful-api-guidelines/models/problem-1.0.1.yaml#/Problem"
```

```rust
pub type PostKudosRequest = PostKudo;
type Created = ();
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
pub enum PostKudosResponse {
    Created(Created),
    Default(Default_),
}
#[async_trait]
pub trait Api {
    async fn post_kudos(request_body: PostKudosRequest) -> PostKudosResponse;
}
```

### Models

Types defined in the `#/components/schemas` section of the document, as well as types defined inline elsewhere in the document, are exported.

#### Name Override

Types defined with a `title` parameter use that to override the derived name. This name is still subject to admendment for deconfliction, if required.

```yaml
title: count
type: integer
```

```rust
type Count = i64;
```

#### Public Type Definitions

Under the hood, this code generates a large quantity of type definitions. By default, these are private, beacuse this tool's output isn't intended to be edited by humans, and `rustdoc` is smart enough to unpack these.

However, it may sometimes be desirable to make the typedef public for a given type. In that case, use the `x-pub-typedef` extension for that item.

```yaml
title: count
type: integer
x-pub-typedef: true
```

```rust
pub type Count = i64;
```

#### `trait CanonicalForm`

Primitive types, and newtypes around primitive types, all have a canonical form when expressed as a string.

This is expressed as `trait CanonicalForm`. This is automatically implemented for all supported primitives and newtypes around primitives. Implementations are encouraged to use this trait to express parsing and expressing these values at the network boundary.

#### Newtypes

Types defined with the `x-newtype: true` extension are wrapped in a newtype. The minimum set of derives is:

- `Debug`
- `Clone`
- `Copy` if the contained type is known to be `Copy`
- `PartialEq`
- `Eq` if the contained type is known to be `Eq`
- [`serde::Serialize`]
- [`serde::Deserialize`]
- [`derive_more::Constructor`]
- [`derive_more::Display`]
- [`derive_more::FromStr`]

`x-newtype` may optionally be an object instead of a boolean. In that case, it supports these fields, all of which default to false:

```yaml
x-newtype:
  from:      true  # derive `derive_more::From`
  into:      true  # derive `derive_more::Into`
  deref:     true  # derive `derive_more::Deref`
  deref-mut: true  # derive `derive_more::DerefMut`
  pub:       true  # mark the inner item as `pub`
```

- [`derive_more::From`]
- [`derive_more::Into`]
- [`derive_more::Deref`]
- [`derive_more::DerefMut`][`derive_more::Deref`]

[`serde::Serialize`]: https://docs.rs/serde/latest/serde/trait.Serialize.html
[`serde::Deserialize`]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
[`derive_more::Constructor`]: https://jeltef.github.io/derive_more/derive_more/constructor.html
[`derive_more::Display`]: https://jeltef.github.io/derive_more/derive_more/display.html
[`derive_more::FromStr`]: https://jeltef.github.io/derive_more/derive_more/from_str.html
[`derive_more::From`]: https://jeltef.github.io/derive_more/derive_more/from.html
[`derive_more::Into`]: https://jeltef.github.io/derive_more/derive_more/into.html
[`derive_more::Deref`]: https://jeltef.github.io/derive_more/derive_more/deref_mut.html

##### Example

```yaml
components:
  parameters:
    X-REQUEST-ID:
      title: X-Request-ID
      in: header
      description: A custom header that traces a request
      required: true
      schema:
        type: string
        format: uuid
        newtype: true
        newtypeInto: true
      example: 83bbfd48-440f-4648-95a5-278b9d755730
```

```rust
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Constructor,
    derive_more::Display,
    derive_more::FromStr,
    derive_more::Into,
)]
pub struct XRequestId(openapi_gen::reexport::uuid::Uuid);
```

#### Numbers

Numbers are converted into Rust types according to the following scheme:

| **`type`** | **`format`** | Rust type |
|------------|--------------|-----------|
| number     | -            | `f64`     |
| number     | float        | `f32`     |
| number     | double       | `f64`     |
| integer    | -            | `i64`     |
| integer    | int32        | `i32`     |
| integer    | int64        | `i64`     |

##### `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`

These attributes are only available with the `integer-restrictions` feature.

If these attributes are set on an `integer` type, then the Rust type produced is [`bounded_integer::BoundedI32`] or [`bounded_integer::BoundedI64`] as appropriate according to its `format`.

[`bounded_integer::BoundedI32`]: https://docs.rs/bounded-integer/latest/bounded_integer/struct.BoundedI32.html
[`bounded_integer::BoundedI64`]: https://docs.rs/bounded-integer/latest/bounded_integer/struct.BoundedI64.html

##### `multipleOf`

This attribute is only available with the `integer-restrictions` feature.

This is meaningless for floating point numbers due to parse/display inaccuracy and will be ignored for those cases.

If this attribute is set on an integer, a newtype will always be generated, even if one has not otherwise been explicitly requested. It is checked on `CanonicalForm::validate` and `CanonicalForm::canonicalize`. However, **there is no type-level enforcement of this restriction**.

#### Strings

Some string formats are well-known and automatically converted into appropriate Rust types:

| **`type`** | **`format`** | Rust type  | Notes |
|------------|--------------|------------|-------|
| string     | -            | `String`   | Always only contains valid `utf8` encoded data |
| string     | binary       | `Vec<u8>` | unencoded binary data; no canonical form |
| string     | byte         | `Bytes` | base64-encoded binary data (requires feature `bytes`) |
| string     | base64       | `Bytes` | base64-encoded binary data (requires feature `bytes`) |
| string     | date         | `time::Date` | [RFC 3339, section 5.6] (2017-07-21) |
| string     | date-time    | `time::OffsetDateTime` | [RFC 3339, section 5.6] (2017-07-21T17:32:28Z) |
| string     | ip           | `std::net::IpAddr`   | |
| string     | ipv4         | `std::net::Ipv4Addr` | [RFC 791] |
| string     | ipv6         | `std::net::Ipv6Addr` | [RFC 4291] |
| string     | uuid         | `uuid::Uuid`         | [RFC 4122] (requires feature `uuid`) |
| string     | mime         | `mime::Mime`         | [RFC 2045] |
| string     | content-type | `mime::Mime`         | [RFC 2045] |
| string     | accept-header| `accept_header::Accept` | [RFC 9110] |
| string     | _other_      | `String`  | Always only contains valid `utf8` encoded data |

[RFC 3339, section 5.6]: https://tools.ietf.org/html/rfc3339#section-5.6
[RFC 791]: https://tools.ietf.org/html/rfc791
[RFC 2045]: https://datatracker.ietf.org/doc/html/rfc2045
[RFC 4291]: https://tools.ietf.org/html/rfc4291
[RFC 4122]: http://tools.ietf.org/html/rfc4122
[RFC 9110]: https://datatracker.ietf.org/doc/html/rfc9110

##### `pattern`

This attribute is only available with the `string-pattern` feature.

If `pattern` is set, a newtype is generated even if it is not explicitly requested. It is checked on `CanonicalForm::validate` and `CanonicalForm::canonicalize`. However, **there is no type-level enforcement of this restriction**.

Note that the regex language is [specified](https://swagger.io/docs/specification/data-models/data-types/#pattern) to match [ECMA 262] syntax. This implementation uses [`regress`](https://docs.rs/regress/latest/regress/#supported-syntax) to construct regular expressions based on that syntax, _not_ the more common [`regex` crate](https://docs.rs/regex/latest/regex/), which defines Rust-specific syntax.

[ECMA 262]: https://www.ecma-international.org/ecma-262/5.1/#sec-15.10.1

#### Booleans

| **`type`** | Rust type |
|------------|-----------|
| boolean    | `bool`    |

#### Nulls

OpenAPI 3 does not have an distinct `null` type, contrary to JSON Schema. Instead, there are two distinct ways to specify nullability:

- setting `nullable: true` on a type definition.
- omitting an item from the `required` list in a containing object.

These have distinct and independent code generation effects, as shown.

```yaml
Foo:
  type: object
  properties:
    not_nullable_and_required:
      type: integer
    not_nullable_and_not_required:
      type: integer
    nullable_and_required:
      type: integer
      nullable: true
    nullable_and_not_required:
      description: note that this produces an `Option<Option<_>>`
      type: integer
      nullable: true
  required:
    - not_nullable_and_required
    - nullable_and_required
```

```rust
type NotNullableAndRequired = i64;
type NotNullableAndNotRequired = i64;
type MaybeNullableAndRequired = Option<NullableAndRequired>;
type NullableAndRequired = i64;
type MaybeNullableAndNotRequired = Option<NullableAndNotRequired>;
type NullableAndNotRequired = i64;

pub struct Foo {
    pub not_nullable_and_required: NotNullableAndRequired,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_nullable_and_not_required: Option<NotNullableAndNotRequired>,
    pub nullable_and_required: MaybeNullableAndRequired,
    ///note that this produces an `Option<Option<_>>`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable_and_not_required: Option<MaybeNullableAndNotRequired>,
}
```

#### Arrays

`type: array` generally maps to a `Vec<T>`.

##### `minItems`, `maxItems`

These attributes are ignored.

##### `uniqueItems`

If `uniqueItems: true`, then instead of a `Vec<T>`, a `HashSet<T>` is generated.

#### Objects

`type: object` maps to a `struct` or a `HashMap`.

Objects with no `additionalProperties` defined create a `struct`.

```yaml
title: foo
type: object
properties:
    bar:
        type: integer
required:
    - bar
```

```rust
struct Foo {
    bar: i64,
}
```

Properties which are not in the required set are mapped to an `Option`.

```yaml
title: foo
type: object
properties:
    bar:
        type: integer
```

```rust
struct Foo {
    bar: Option<i64>,
}
```

Objects which have no defined `properties` and have `additionalProperties` defined map to a `HashMap`.

```yaml
title: foo
type: object
additionalProperties:
    type: integer
```

```rust
pub type Foo = HashMap<String, i64>;
```

The key type of a `HashMap` is always a `String`.

Objects with both `properties` and `additionalProperties` are forbidden.

##### `minProperties`, `maxProperties`

These attributes are ignored.

##### `readOnly`, `writeOnly`

**Note**: the designations here are from the perspective of a client. Their meaning is inverted in the server context.

If `readOnly: true` is set on a property, the server is expected to emit it, but never read it. This corresponds to `#[serde(skip_deserializing)]`.

If `writeOnly: true` is set on a property, the server is expected to read it, but never emit it. This corresponds to `#[serde(skip_serializing)]`.

```yaml
title: auth
type: object
properties:
  id:
    # Returned by GET, not used in POST/PUT/PATCH
    type: integer
    readOnly: true
  username:
    type: string
  password:
    # Used in POST/PUT/PATCH, not returned by GET
    type: string
    writeOnly: true
```

```rust
pub struct Auth {
    #[serde(skip_deserializing)]
    id: Option<i64>,
    username: Option<String>,
    #[serde(skip_serializing)]
    password: Option<String>,
}
```

#### Missing Types

A schema without a type specified produces a `serde_json::Value`.

```yaml
title: anyValue
description: "no type specified, so 🤷"
```

```rust
/// no type specified, so 🤷
pub type AnyValue = serde_json::Value;
```

These can also be nullable, but only explicitly:

```yaml
title: anyValue
nullable: true
```

```rust
pub type AnyValue = Option<serde_json::Value>;
```

#### Primitive Enums

In OpenAPI an enum is a constraint the permitted values of an otherwise primitive type.

This generator supports only enums with `type: string`.

String enums produce unit Rust enums.

```yaml
title: sort
description: Sort order
schema:
  type: string
  enum: [asc, desc]
```

```rust
/// Sort order
pub enum Sort {
    Asc,
    Desc,
}
```

##### Nullable Enums

Because the OpenAPI spec considers enums to be a constraint distinct from other constraints, nullable enums _must_ include `null` among the permitted variants. [Ruling](https://github.com/OAI/OpenAPI-Specification/blob/main/proposals/2019-10-31-Clarify-Nullable.md#if-a-schema-specifies-nullable-true-and-enum-1-2-3-does-that-schema-allow-null-values-see-1900).

If an enum definition includes `nullable: true`, then this generator will produce an `Option` whether or not the variant list includes `null`. This is an intentional deviation from the specification.

If an enum definition includes `nullable: true`, then this generator will strip `null` from the list of emitted variants.

```yaml
title: sort
schema:
  type: string
  enum: [asc, desc, null]
nullable: true
```

```rust
pub enum Sort {
    Asc,
    Desc,
}

pub type MaybeSort = Option<Sort>;
```

#### `oneOf`

The `oneOf` keyword allows us to combine several schemas, matching only one. It is an untagged union.

Note that due to technical limitations, the generated deserializer does not actually ensure that the supplied JSON only matches one of the provided options. Instead, it just returns the first variant whose deserializer successfully deserializes the supplied JSON.

Note that we're leaving off most derives for simplicity in this example.

```yaml
paths:
  /pets:
    patch:
      operationId: updatePet
      requestBody:
        content:
          application/json:
            schema:
              oneOf:
                - $ref: '#/components/schemas/Cat'
                - $ref: '#/components/schemas/Dog'
      responses:
        '200':
          description: Updated
components:
  schemas:
    Dog:
      type: object
      properties:
        bark:
          type: boolean
        breed:
          type: string
          enum: [Dingo, Husky, Retriever, Shepherd]
    Cat:
      type: object
      properties:
        hunts:
          type: boolean
        age:
          type: integer
```

```rust
pub enum Breed {
    Dingo,
    Husky,
    Retriever,
    Shepherd,
}

pub struct Dog {
    bark: Option<bool>,
    breed: Option<Breed>,
}

pub struct Cat {
    hunts: Option<bool>,
    age: Option<i64>,
}

#[serde(untagged)]
pub enum UpdatePetRequest {
    Dog(Dog),
    Cat(Cat),
}
```

#### `oneOf` with `discriminator`

The `oneOf` keyword allows us to combine several schemas, and the `discriminator` keyword specifies precisely which one is matched. It is a tagged union.

Note that we're leaving off most derives for simplicity in this example.

```yaml
Pet:
  oneOf:
    - "$ref": "#/components/schemas/Dog"
    - "$ref": "#/components/schemas/Cat"
  discriminator:
    propertyName: petType
```

```rust
#[serde(tag = "petType")]
pub enum Pet {
    Dog(Dog),
    Cat(Cat),
}
```

Note the JSON matched by the above _must_ contain a field `petType` which has the either the value `"Dog"` or the value `"Cat"`.

It is possible to specify an explicit mapping, in case it is impossible or suboptimal to use the derived name for the variants, it is valid to supply a mapping. This mapping does not need to be complete.

```yaml
Pet:
  oneOf:
    - "$ref": "#/components/schemas/Dog"
    - "$ref": "#/components/schemas/Cat"
  discriminator:
    propertyName: petType
    mapping:
      woofer: "#/components/schemas/Dog"
```

```rust
#[serde(tag = "petType")]
pub enum Pet {
    #[serde(rename = "woofer")]
    Dog(Dog),
    Cat(Cat),
}
```

#### `allOf` Singletons for Property Overrides

OpenAPI defines [several schema properties](https://swagger.io/docs/specification/data-models/keywords/) which can apply to any schema. This means that it is possible to define a schema like:

```yaml
components:
  schemas:
    Thing:
      type: object
      properties:
        id:
          type: string
          format: uuid
          readOnly: true

    WriteableThing:
      type: object
      properties:
        id:
          type: string
          format: uuid
      required:
        - id
```

However, because these properties apply at the schema level instead of the object field level, we run into problems if we want to i.e. reuse the type to reduce duplication. This schema is invalid:

```yaml
components:
  schemas:
    Id:
      type: string
      format: uuid

    Thing:
      type: object
      properties:
        # invalid! When a reference is specified, field properties are ignored
        id:
          readOnly: true
          $ref: "#/components/schemas/Id"
```

We can work around this and define field-level property overrides by using an `allOf` singleton. This pattern merges certain additional properties with a referenced schema, at the field level. It looks like this:

```yaml
    Thing:
      type: object
      properties:
        id:
          readOnly: true
          allOf:
            - $ref: "#/components/schemas/Id"
```

The rules for an `allOf` singleton are as follows:

- the schema is a property sub-schema of an object type
- the schema is not in the `required` list of the object type
- the schema has an `allOf` definition
- the `allOf` definition possesses exactly one item
- the `allOf` item is a reference
- the schema possesses 0 or more properties from this list:

  - `readOnly`
  - `writeOnly`

Additional properties may be supported in the future, but for now, properties outside this list are ignored.

When the schema meets these rules, the `allOf` singleton is used to assign field-level properties without affecting the referenced object. The output from the (implied) schema defined above is:

```rust
pub type Id = Uuid;

pub struct Thing {
    #[serde(skip_deserializing)]
    id: Option<Id>,
}

pub struct WriteableThing {
    id: Id,
}
```

#### `allOf` for Merging Object Definitions

This schema combinator is not currently supported, but might be supported in the future.

#### `anyOf`, `not`

These schema combinators are not supported and are unlikely to receive support in the future. They do not map cleanly to Rust's data model.

Recommended workaround: define the schema without these combinators.
