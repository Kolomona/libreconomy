- Is libreterra using ECS?
- Water is swimmable


- Time
    - We should have a unit of time that maps on to the real world. This should be a multiplier variable in config.js . I'm not sure how to properly represent this but 1 second of real world time should represent 30 minutes of simulation time.
    - all time based variables should respect this and be realistic. Example a human can go 3 days without water before dying of thirst, 30 days without food before dying of hunger, etc. Every entity will have different ability numbers based on their type.

- Movement
   - Entities should have different speeds. Walking running, sprinting. Each faster speed costs more energy and increases tiredness.
   - Decisions regarding what speed to move at should be based on their current circumstances.

- Swimming
    - entity should have a canSwim attribute
    - entity should have a swim multiplier to it's speed. For example a human may move at .25 * maxSpeed whereas a frog may move at 2 * maxSpeed
    - entity should have a swim energy multiplier. For example a human may use 3 times more energy swimming while a frog may use 50% of energy to swim
    - an entity with tiredness over 95% that is swimming will drown


- Drinking
    - When an entity is at the shore drinking they should drink until their thirst goes to zero
    
 
 - Pain Response
     - When an entity is hungry, thirsty or tired it causes a certain amount of pain. The entity should make decisions about what to do based on the pain response they have. Example: If a human is pained more by hunger than thirst he should search for food before he searches for water.
     - Some things pain more than others. Thirst pains more than hunger, hunger pains more than tiredness. After all an animal can go without food for longer than it can go without water

- Tiredness and sleeping
    - An entity should have a sleepTimeFactor where x percentage of rest restored for every clock tick. Some entities may require more sleep to be fully rested.
    - When an entity chooses to sleep then it will sleep until it is fully rested or it's sleep is interrupted
    - When an entity passes out from exhaustion it will only sleep until 60% of it's tiredness is restored. This is a penalty to the entity for not managing sleep properly.
    - When an entity is sleeping it's hunger and thirst will accumulate at 10% of the normal rate of accumilation.

- Grass lifecycle
    - dirt will grow grass after a lng per