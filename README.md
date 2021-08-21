# Ota

![build and test](https://github.com/j16r/ota/actions/workflows/rust.yml/badge.svg)

Ota is a self contained web application that functions a little like a content
management system with a low level templating language and content addressing
query language.

It is intended to be used for smaller sites as a blogging platform, or simple
web shop.

## Goals

 1. Self contained and easy to deploy.
 2. All changes are committed to local directory.
 3. Simple web server.
 4. Basic authentication system.
 5. Very little structure.

## Walkthrough

The root document in Ota is the index page, when you first load it, you get a
default article with a splash screen and little else. Like the default nginx
start page.

Once you authenticate, you will be able to interact with a basic text editor
which can edit the root template. Within this you get access to handlebars
templates, and built in helper functions.

The built in helper functions will allow you to access other articles,
selecting them by a unique ID, by topic tags
