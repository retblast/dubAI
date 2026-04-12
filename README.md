# dubAI 

AI-powered dubbing toolset that I am working on. It's mostly for my own usage but I try to make it appropiate enough for other people to use, too.

# TODO
- Generate SRT
- Translate SRT [partially done]
- Dub according to SRT [wip]
  - [x] Get SRT fragment to know timing, position and text 
  - [x] Create n audio files according to the SRT timings
  - [x] Send i audio file together with its corresponding text to the dubber LLM, n times
  - Save the files
  - All of this in a proper folder
- Mix original audio with dubbed audio
  - Duck original audio's volume according to the SRT time fragments
  - Adjust audio as needed

  Note: I have to have koboldcpp running before the voice references are created! Also, decide whether I will
- Pass LLM addresses thru CLI

#  Principles
- Modular
  
  You should be able to use any tool you want for each step of the process, if possible.
