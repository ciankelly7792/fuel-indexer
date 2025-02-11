# Foreign Keys

- The Fuel indexer service supports foreign key constraints and relationships using a combination of GraphQL schema and a database.
- There are two types of uses for foreign keys - _implicit_ and _explicit_.

> IMPORTANT:
>
> Implicit foreign keys do not require a `@join` directive. When using implicit foreign key references, merely add the referenced object as a field type (shown below). A lookup will automagically be done to add a foreign key constraint using this object's' `id` field.
>
> Note that implicit foreign key relationships _only_ use the `id` field on the referenced table. If you plan to use implicit foreign keys, the object being referenced _must_ have an `id` field.
>
> In contrast, explicit foreign keys _do_ require a `@join` directive. Explicit foreign key references work similarly to implicit foreign keys; however, when using explicit foreign key references, you must add a `@join` directive after your object type. This `@join` directive includes the field in your foreign object that you would like to reference (shown below).

Let's learn how to use each foreign key type by looking at some GraphQL schema examples.

## Usage

### Implicit foreign keys

```graphql
type Book @entity {
    id: ID!
    name: Bytes8!
}

type Library @entity {
    id: ID!
    book: Book!
}
```

#### Implicit foreign key breakdown

Given the above schema, two entities will be created: a `Book` entity, and a `Library` entity. As you can see, we add the `Book` entity as an attribute on the `Library` entity, thus conveying that we want a one-to-many or one-to-one relationship between `Library` and `Book`. This means that for a given `Library`, we may also fetch one or many `Book` entities. It also means that the column `library.book` will be an integer type that references `book.id`.

### Explicit foreign keys

```graphql
type Book @entity {
    id: ID!
    name: Bytes8! @unique
}

type Library @entity {
    id: ID!
    book: Book! @join(on:name)
}
```

#### Explicit foreign key breakdown

For the most part, this works the same way as implicit foreign key usage. However, as you can see, instead of implicitly using `book.id` as the reference column for our `Book` object, we're instead explicitly specifying that we want `book.name` to serve as our foreign key. Also, please note that since we're using `book.name` in our foreign key constraint, that column is required to be unique (via the `@unique` directive).
