<!--- #!#tep:(concept) -->
<!--- #!#tep:[concept](concept) -->
# Concept

## Short definition

`tep` is a tool for connecting meaningful things in text.

It does that through three main building blocks:
- **entities** — logical things you care about
- **anchors** — named anchor points identified by tags in files
- **links** — relationships between entities

## The mental model

### Entity

An **entity** is the logical thing itself.

It is not the file tag. It is the concept or object you want to track.

Examples:
- an API endpoint
- a DTO
- a service
- a feature
- a character in a book
- a research claim

Think of the entity as:
> "the thing I want to refer to consistently."

An entity can exist even if it appears in many places.

### Anchor

An **anchor** is a named bridge point placed as a tag in a file.

Example:
```java
// #!#tep:[student_processor](student) #tepignore
```

Important:
- the name after `tep:` is the **anchor identity**
- the tag also carries an entity reference list
- one anchor can be connected to multiple entities
- one entity can be connected to multiple anchors

Think of the anchor as:
> "a bridge point between text and the logical graph."

The anchor connects a real location in a file to one or more entities.

### Link

A **link** is a directional relationship between two entities.

Examples:
- endpoint **uses** dto
- endpoint **calls** service
- feature **is-documented-by** design-doc
- chapter **mentions** character

A link is directional: from one entity to another. It also carries a free-text relation description.

Think of the link as:
> "how one logical thing is connected to another logical thing."

## Why split these concepts?

- **entity** = the thing
- **anchor** = a tagged bridge point in text
- **link** = a logical relation between things

That lets `tep` connect the physical world of files with the logical world of concepts.

## Many-to-many model

### Entities to anchors
- one entity can point to many anchors
- one anchor can point to many entities

A single location in code or docs may be relevant to several concepts, and one concept may appear in many files.

### Entities to entities
- entities can also link to other entities
- links are directional

## Example

Imagine a method that converts a student into a basic user.

One anchor tag sits near that code location, referencing both entities:
```java
// #!#tep:[student_converter](student,basic_user) #tepignore
```

Separately, those entities may also link to other entities in the graph.

This avoids stacking multiple anchor tags at the same location.

## Location metadata

Anchors store file path, line, and offset as metadata.

But these are not durable identity. Files move, lines shift.

The durable identity is the anchor name in the tag.

## What tep is not

`tep` is not:
- version control
- a full knowledge-graph platform
- an IDE replacement
- a magical semantic engine

Its job:
- store entities
- discover anchor tags
- store entity-anchor connections
- store entity links
- answer graph and retrieval queries
