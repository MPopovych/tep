# Concept

## Short definition

`tep` is a tool for connecting meaningful things in text.

It does that through three main building blocks:
- **entities** — logical things you care about
- **anchors** — anchor points identified by tags in files
- **links** — relationships between entities

## The mental model

### Entity

[#!#1#tep:12](entity)

An **entity** is the logical thing itself.

It is not the file tag.
It is the concept or object you want to track.

Examples:
- an API endpoint
- a DTO
- a service
- a feature
- a character in a book
- a chapter
- a design concept
- a research claim

Think of the entity as:
> “the thing I want to refer to consistently.”

An entity can exist even if it appears in many places.

### Anchor

[#!#1#tep:13](anchor,anchor.tag)

An **anchor** is a stable anchor identity represented by a tag placed in a file.

Example:
```java
// [#!#1#tep:123763636473] #tepignore
```

Important:
- the value inside the tag is the **anchor ID**
- the tag does **not** contain an entity ID
- one anchor can be connected to multiple entities
- one entity can be connected to multiple anchors

Think of the anchor as:
> “a bridge point between text and the logical graph.”

The anchor is how tep connects a real location in a file to one or more entities.

### Link

[#!#1#tep:14](link,entity)

A **link** is a directional relationship between two entities.

Examples:
- endpoint **uses** dto
- endpoint **calls** service
- feature **is-documented-by** design-doc
- chapter **mentions** character

A link is directional:
- from one entity
- to another entity

It also carries metadata such as:
- relation type
- priority

Think of the link as:
> “how one logical thing is connected to another logical thing.”

## Why split these concepts?

This separation keeps the model clean.

- **entity** = the thing
- **anchor** = a tagged bridge point in text
- **link** = a logical relation between things

That lets `tep` connect the physical world of files with the logical world of concepts.

## Many-to-many model

This is the currently chosen model.

### Entities to anchors
- one entity can point to many anchors
- one anchor can point to many entities

This is useful because a single place in code or docs may be relevant to several concepts, and one concept may appear in many files.

### Entities to entities
- entities can also link to other entities
- these links are directional and priorityed

This keeps the graph flexible without forcing artificial “pair entities” for every relationship.

## Example

Imagine a method that converts a student into a basic user.

A single anchor tag may sit near that code location.
That one anchor may connect to multiple entities such as:
- `student`
- `basic-user`
- `student-conversion`

Separately, the entities may also link to other entities in the graph.

This avoids clutter from stacking several anchor tags at the same location.

## Location metadata

Anchors may store metadata such as file path, line, or offset.

But an important assumption is:
- locations are dynamic,
- files change over time,
- line and offset data are useful metadata only,
- they should not be treated as fully trustworthy identity.

The true durable identity is the anchor ID in the file tag.

## What tep is not

`tep` is not:
- version control,
- a full knowledge-graph platform,
- an IDE replacement,
- a magical semantic engine.

Its job is simpler:
- store entities,
- discover anchor tags,
- store entity-anchor connections,
- store entity links,
- answer graph and retrieval queries.
