# Purpose of Redesign

## Backend

### Build Standard of the Diagram

The current codebase is mixed with OOP and FP, without a clear line on why one is needed. For ideal flow, FP is the best choice. However, if OOP is needed for a good reason, it should stay, and we accept the category to be OOP.

### Form a Clear Boundary for Core, Service, and Interface

The current service level doesn't have a clear boundary of what should be done and what should not. A service should take parameters from interfaces, call core functions and utils, and finish jobs with a pipeline. If the job is complicated, it can have its own dedicated file for its pipeline. The core and util functions should strictly hold to one purpose only, avoiding monoliths.

### Separate Low-Level FS and Higher-Level Business

In `ModFS`, there are low-level FS operations mixed with high-level business logic. Separating both would give a better view of the logic and business hierarchy.

### Reduce Unintended Distraction

Remove all comments as they are not necessary to understand the context. If they are needed, the code might have grown too complex and needs separation.

### Build Principles for Endpoints

The original endpoint design doesn't follow a specific set of rules. One can set the scope and intent generally, avoiding deviation over a large range.

## Frontend

### Redesign UI

This is the core of this design. The previous design doesn't look coherent enough — it is a mix of Fluent Design and the standard web application aesthetic. In the new design, the contrast should be guaranteed, with or without a transparent background.

The second issue that needs to be addressed is that the header is not fixed, and the scrollbar doesn't look right.

### Set Rules for i18n

AI tends to create new utils all the time, and the same goes for text that needs translating. While `lingui` provides great convenience, we still need to avoid repeated translations for the same content used in different places.

## General

### Audit the Current Codebase and Structure

### Audit Dependencies and Their Necessity

### Logging Fix

The current backend doesn't yield logs correctly, and fatal error on frontend might be eaten.
