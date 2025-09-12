# Doggo

A modern, Cargo-like, build system for C, C++, and ASM. Designed to build C/C++, without the fluff.

## Why a new build system?

Doggo's primary goal is to make building C, C++, and ASM as easy as it is to build Rust.

A secondary goal is to decrease the difficulty of integrating Rust libraries into C/C++ codebases.

## What are your non-goals?

Speed and extensability are explicit non-goals.
Doggo aims to provide a sensible set of defaults, but if you need more complicated build logic, use another build system.

Doggo also won't generate makefiles or Ninja files, and instead will dispatch build commands itself.

Doggo isn't intended to be a replacement for CMake, Ninja, MSVC, XCode, etc. It's intended for new projects that don't need complicated build logic.

## What compiler(s)?

Right now, we only support Clang.
This is to make the burden of maintaining compiler flags smaller, make prototyping faster, and add easier support for cross-compilation, leveraging LLVM's ecosystem.

## Why the name?

Dog + Cargo = Doggo :3

## Project status

The project is currently in the **WIP** stage.
