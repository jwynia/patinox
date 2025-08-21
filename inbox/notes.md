# Stuff I Want To Discuss/Design
* When it comes to memory, context management, etc. vector search is about similarity, not relevance and it relies on correlation to be high enough for that to be a proxy. But, actual linkage is what Google used for decades as a better metric of relevancy. Both should likely be involved when searching through stuff for relevant context. Lexical algorithms matter. https://www.tigerdata.com/blog/vector-search-isnt-the-answer-to-everything-so-what-is-a-technical-deep-dive

* Guessing vs grounding in knowledge or things that COULD be known if just looked for instead of guessing:
Let me run a test generation to the test_wiki folder to demonstrate the variety improvements:

  Bash(cd /workspaces/world-simulator && npm run simulation -- --outputDir test_wiki --genre fantasy)
  ⎿  Error: npm error code ENOENT
     npm error syscall open
     npm error path /workspaces/world-simulator/package.json
     npm error errno -2
     npm error enoent Could not read package.json: Error: ENOENT: no such file or directory, open 
     '/workspaces/world-simulator/package.json'
     npm error enoent This is related to npm not being able to find a file.
     npm error enoent
     npm error A complete log of this run can be found in: /home/node/.npm/_logs/2025-08-21T02_16_22_664Z-debug-0.log

  Bash(npm run simulation -- --outputDir test_wiki --genre fantasy)
  ⎿  Error: npm error Missing script: "simulation"
     npm error
     npm error To see a list of scripts, run:
     npm error   npm run
     npm error A complete log of this run can be found in: /home/node/.npm/_logs/2025-08-21T02_16_29_029Z-debug-0.log

● Bash(npm run dev -- --outputDir test_wiki --genre fantasy)
  ⎿  Error: error: unknown option '--outputDir'


     > sim-world@1.0.0 dev
     > mastra dev --outputDir test_wiki --genre fantasy
  ⎿  Interrupted by user