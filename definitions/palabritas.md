# Palabritas: cuentito's language

`cuentitos` is a narrative engine that includes its own language for writers to create dynamic storytelling: `palabritas`.

In this document I will introduce you to `palabritas`.

## Basics

You can start by writing small pieces of text. Each newline will prompt the user to continue.

```cuentitos
You've just arrived in the bustling city, full of excitement and anticipation for your new job. 
The skyline reaches for the clouds, and the sounds of traffic and people surround you. 
As you take your first steps in this urban jungle, you feel a mix of emotions, hoping to find your place in this new environment.
```

Now, you can add options after an event by indenting with two spaces and starting the line for each option with an asterisk `*`

```cuentitos
[...]
As you take your first steps in this urban jungle, you feel a mix of emotions, hoping to find your place in this new environment.
  * I take a walk through a nearby park to relax and acclimate to the city.
  * I visit a popular street market to experience the city's unique flavors and energy.
```

You can now start a new narrative path by indenting with 2 spaces and writing the next text bits.

```cuentitos
  [...]
  * I take a walk through a nearby park to relax and acclimate to the city.
    As you stroll through the nearby park, the soothing sounds of rustling leaves and chirping birds help calm your senses. 
    You find a quiet bench to sit on, taking a moment to breathe deeply and gather your thoughts. 
    This serene oasis gives you the confidence to face the challenges ahead as you acclimate to your new life in the bustling city.
```

## Probabilistic One-Off's

An interesting feature of cuentitos is that you can use probability to render different things in the story. For example, in our case, we can use probability to have someone sitting on the bench or not.

```cuentitos
    [...]
    You find a quiet bench to sit on, taking a moment to breathe deeply and gather your thoughts. 
    (50%) A solitary figure sits on the bench, engrossed in a book, seemingly unfazed by the surrounding city noise.
    This serene oasis gives you the confidence to face the challenges ahead as you acclimate to your new life in the bustling city.
    [...]
```

This will make it so that half of the time the solitary figure is there sitting there, and half of the time, they're not. Whenever you have a one off text with probability, you can directly write said probability as a percentage. So in this case there is a `50%` chance that this text will be rendered. We call this the `one-off percentage notation`.

Another way is to use the `one-off probability notation`, that uses a range from 0 to 1 to determine how probable that event is. In this case, we'd have to use `0.5`. 

```cuentitos
    [...]
    You find a quiet bench to sit on, taking a moment to breathe deeply and gather your thoughts. 
    (0.5) A solitary figure sits on the bench, engrossed in a book, seemingly unfazed by the surrounding city noise.
    This serene oasis gives you the confidence to face the challenges ahead as you acclimate to your new life in the bustling city.
    [...]
```


In practice, both notations are similar and it's mostly up to you which one to use, the only difference is the precission: percentage notation only supports integers, while probability notation supports real numbers (floats). You can't do `55.5%`, but you can do `0.555`.

You could add options on probabilistic branches, but let's keep going for now.

## Probabilistic Buckets

Another interesting feature is the ability to create `probabilistic buckets`.

A bucket is a set of probable paths where the engine will pick one every time is asked to render that bucket. Let's do a probabilistic bucket with the second option.

```cuentitos
  * I visit a popular street market to experience the city's unique flavors and energy.
    (50%) At the bustling street market, you discover a food stand offering mouthwatering delicacies. 
      As you sample the delicious fare, you strike up a conversation with the enthusiastic vendor.
      It's a welcome distraction from the noise around you.
    (20%) As you try to navigate the crowded market, you're drawn to the entrancing melody of a street musician. 
      The captivating sound creates a soothing bubble, momentarily transporting you away from the city's noise. 
      You take a moment to appreciate the beauty of the music, feeling a connection to the artist and the vibrant energy they bring to the urban landscape.
    (30%) Wandering the market, you stumble upon a hidden alley adorned with vibrant street art. 
      Each colorful mural tells a different story, capturing your imagination and sparking your creativity. 
      This unexpected oasis of visual tranquility provides a respite from the chaos of the city, inspiring you to explore more of the urban canvas and the stories it holds.
```

In this example, we added an indentation from the option and ten added 3 narrative paths with probabilities expressed in percentage notation. Every time you indent and add probabilities you're effectively creating a bucket.

Notice that the sum of the probabilities is `100%`. This is a requirement if using `percentage notation` in a bucket, the compiler will fail if you have a different sum. The same will happen if you use `probability notation`, the sum has to be `1`, otherwise the compiler will throw an error.

There is another useful notation available on buckets that is called the `frequency notation`, where you can set the frequency of each event. The probability will be calculated as the frequency over the sum of the frequencies of all the events in the bucket. There is no requirement to keep a consistent sum of frequencies, the probability of all events will adjust as you add more or change the existing ones.

```cuentitos
  * I visit a popular street market to experience the city's unique flavors and energy.
    (50) At the bustling street market, you discover a food stand offering mouthwatering delicacies. 
      As you sample the delicious fare, you strike up a conversation with the enthusiastic vendor.
      It's a welcome distraction from the noise around you.
    (2) As you try to navigate the crowded market, you're drawn to the entrancing melody of a street musician. 
      The captivating sound creates a soothing bubble, momentarily transporting you away from the city's noise. 
      You take a moment to appreciate the beauty of the music, feeling a connection to the artist and the vibrant energy they bring to the urban landscape.
    (500) Wandering the market, you stumble upon a hidden alley adorned with vibrant street art. 
      Each colorful mural tells a different story, capturing your imagination and sparking your creativity. 
      This unexpected oasis of visual tranquility provides a respite from the chaos of the city, inspiring you to explore more of the urban canvas and the stories it holds.

```
Given what we just said, which event is more probable of the three?

The hidden alley! Let's see why:

The sum of frequencies is `50 + 2 + 500 = 552`, then the probability of the deli stand is `50 / 552 ~= 0.09`, the probability of the musician is `2 / 552 ~= 0.003` and the probability of the hidden alley is `500 / 552 ~= 0.9`. So 9 out of 10 times we'll get the alley.

### Named Buckets

You can also create what we call `named buckets`. These buckets support probabilities, conditions, modifiers as any piece of text.

```cuentitos
    (50) At the bustling street market, you discover a food stand offering mouthwatering delicacies. 
      [morning_vendor]
        req time_of_day morning
        [(100) happy_vendor]
          (50%) You notice the stand owner, their eyes sparkling with joy as they animatedly describe their homemade offerings to an eager customer.
          (50%) You see the owner beaming with joy, their infectious smile and animated gestures inviting customers to try their delectable creations.
        [(25) tired_vendor]
          (50%) You come across a vendor with furrowed brows and a tense expression, their voice raised as they heatedly argue with a customer over a transaction at their stand.
          (50%) You spot a visibly agitated vendor, their clenched fists and piercing glare making it clear that they're unhappy with the current situation unfolding before them.
      [night_vendor]
        req time_of_day night
        (50%) You observe a vendor at a small food stand, their shoulders slumped and eyes slightly glazed as they quietly serve customers, mustering just enough energy to complete each transaction.
        (50%) The vendor at a nearby food stand appears worn, their movements slow and deliberate, as they attempt to maintain a smile while attending to the seemingly endless stream of customers.
      You feel they're too busy to bother them with questions.
```

To create one of these you wrap the name and probability with `[]`, for example `[(50%) my_name]`. The name must to be snake case (lower_case_and_underscored). Then you can apply `req`, `set` or `set`


## Probability of Options

Probability can be applied to pretty much everything in `cuentitos`, including options. Let's explore that.

You can add a chance for an option to show up by adding the probability after the asterisk:

```cuentitos
      [...]
      You take a moment to appreciate the beauty of the music, feeling a connection to the artist and the vibrant energy they bring to the urban landscape.
        * (50%) I leave two dollars to the artist.
          The musician smiles and nods thanking you.
        * I nod when the musician looks my way, to show I really enjoy the music.
      [...]
```

If a choice doesn't have probability set, the probability of it showing up is 100%.

In this case there is a 50% chance the user will see the option to leave two dollars to the musician. So half of the times he'll support the artist economically, and half just nod.

### Options bucket

If all the options in an indentation level are probabilistic, then a bucket is created.

This means that only one of those options will show up, respecting the probability rules.

```cuentitos
    (500) Wandering the market, you stumble upon a hidden alley adorned with vibrant street art. 
      Each colorful mural tells a different story, capturing your imagination and sparking your creativity. 
        * (50%) I grab my phone and take pictures of the murals.
        * (50%) I keep walking, even if the murals look good, the darkness of the alley is unsettling.
```

In this case, only one of the options will show up, each one with a 50% chance.

## Reacting to State

Up until this point, we've not used the game state to setup conditions. So let's do that now. We'll start writing from the root indentation level again. This texts will show up when the previous section is navigated until there is no more text to show. It will automatically backtrack to the root if no texts display in any indentation level.

```cuentitos
Feeling mentally and physically exhausted from the day's adventures, you decide it's time to head back to your hotel.
As you enter the peaceful sanctuary of your room, you take a deep breath, relieved to have a quiet space where you can recharge and prepare for the challenges ahead.

The sun shines bright through the window.
  req time_of_day !night
The moonlight gives the room a peaceful tone.
  req time_of_day night
```

In this case, we're putting requirements to these two lines of text using the `req` command. The lines will only show up if the `time_of_day` variable value satisfies the requirements. The first one will show if time of day is not `night`, the second one, will show when time of day is `night`.

Check the `Configuration` section to learn about variables.

In this case, the requirements are mutually exclusive, only one line will show at a time. But that's not mandatory. You could have multiple lines with different requirements and have them show one after the other if the conditions are met.

```cuentitos
[...]
The sun shines bright through the window.
  req time_of_day !night
The moonlight gives the room a peaceful tone.
  req time_of_day night
You start to think about all the stuff you need to do tomorrow.
  req time_of_day_night
That makes you feel overwhelmed.
  req time_of_day_night
  req energy <10
You decide to focus on the now...
  req time_of_day_night
```

Thse three lines will only show at night, after `The moonlight gives...`. The second one only if the player is low on energy.

You can also add requirements to options.

```cuentitos
[...]
You decide to focus on the now...
  req time_of_day_night
  * I make some tea
    req item tea
    A good cup of tea is always good to regulate after the sensory overload of the city.
  * I go to bed
    Feeling depleted of spoons, you go right back to bed.
  * I call my parents
    req energy >10
```

In this case, to make tea you need an `item` with identifier `tea` in the inventory.
You also need more than `10` points of energy to call your parents.

### Conditional probability changes

You can change the probability of a text of option if a given condition is met by using the `freq` command.

```cuentitos
[...]
  * I call my parents
    req energy >10
    (30) The phone rings twice, and dad picks up.
      req energy>20
    (10) The phone rings twice, and mom picks up.
      req energy>20
      freq time_of_day night 100
    (40) The phone rings ten times, nobody is at home.
```

Let's analyse this situation:

 * If the player has `energy` between `11` and `20`, nobody picks up, because the condition of `energy>20` is not met, so mom and dad will not pickup.
 * If `energy` is `21` or more and `time_of_day` is not `night`, there are more chances that dad will pick up than mom (frequency `30` for dad, vs `10` for mom).
 * If `energy` is `21` or more and `time_of_day` is `night`, it is very likely that mom will pick up, because the frequency is `110` (vs `30` for dad, and `40` for no one).

In this way we can create very complex condition sets and frequency alterations that we can use to modify the behavior of our story. This is incredibly useful to create games that feel alive, in this example, the mother of the player works all day outside the house, while the dad takes care of the home during the day. 

Conditions and probability modifications can be used to express rules of the world and its randomness to successfully represent a world that is more complex, alive and random than simple rule-based ones.

The `freq` command can take negative numbers too, to reduce the frequency instead of increasing it.

```cuentitos
    (10) The phone rings twice, and mom picks up.
      req energy>20
      freq time_of_day !night -100
```

If the frequency ends up being 0 or less, the affected element will not be considered. In this case, mom will not pickup the phone at all unless it's night time.

## Changing State

The next thing we want to talk about is modifying state.

`cuentitos`' runtime will manage the state for all the variables that get defined in the `Configuration`. We can make changes to that state from the game engine or from the story definition itself.

```cuentitos
  * I go to bed
    Feeling depleted of spoons, you go right back to bed.
    set energy 10
    set day +1
    set time -7.5
``` 

For that we use the `set` command with the variable name (in this case `energy`, `day` and `time`) and the value modification we want to apply. In this case, we set `energy` to `10`, added `1` to `day`, and subtracted `7.5` from `time`. From this we can infer that `energy` and `day` are variables of the `integer` type and `time` is of the `float` type. We also support `bool`, `enum` and `string` types. Check the `Configuration` section below for more details.
Also, you can add `+`, subtract `-`, multiply `*` and divide `/` your integers and floats!

## Sections, diverts and subsections

Being heavily inspired by [Ink](https://www.inklestudios.com/ink/), we shamelessly stole the idea of `knots`(`sections`), `diverts` and `stitches` (`subsections`) from there.

A `section` is a part of the story that you assign a name to so that you can move to it easily. You define sections by wrapping a `snake_case` identifier with a hash simbol (`#`), as you'd do in markdown.

```cuentitos
# second_day
You wake up feeling refreshed.
```

Then you can go to a section by using the arrow (`divert`) command `->`.

```cuentitos
  * I go to bed
    Feeling depleted of spoons, you go right back to bed.
    set energy 10
    set day +1
    set time -7.5
    -> second_day
```

Once the story hits `-> second_day`, the player will be directed to that section.

A `subsection` is a section within a `section` and it's defined by using multiple hashes `##`, `###`, etc.
You can access a subsection by using the name of the section, then a slask and then the subsection name (`second_day/museum` in the example below).

If the subsection is within the current section, you can ignore the section name (`farmers_market` in the example below).

```cuentitos
## second_day
  You wake up feeling refreshed. Let's see what this day brings.
    * Explore a museum
      -> second_day/museum
    * Go to the Farmer's Market
      -> farmers_market

## museum
  You get to the museum door. You watch through the window. It seems crowded.

## farmers_market
  You get to the farmer's market. It's very early and some stands are still being set up.
```

You can also add requirements and probabilities to sections and sub-sections 
themselves as if they were any piece of text.

For example:
```cuentitos
## second_day
  req day 2
  You wake up feeling refreshed. Let's see what this day brings.
```

### Finishing the game
`-> END`

### Boomerang divert

Another way to access a section is using the `boomerang divert` command `<->`. Unlike the regular `divert`, the `boomerang divert` command will take you back to your original spot once you're done with the section. 

```cuentitos
  * I go to bed
    Feeling depleted of spoons, you go right back to bed.
    set energy 10
    set day +1
    set time -7.5
    <-> second_day
  
    At last the end has come.
    -> END
  
# second_day
  You wake up feeling refreshed. Let's see what this day brings.
```

In this example, once the section second_day finishes, the story goes back to where the `boomerang divert` was and shows the line `At last the end has come.`.

### Unique command

To make sure that the story never reaches the same spot twice, you can use the `unique` command.

```cuentitos
# quest_giver
  You've come seeking a brave quest.
    -> quest_pool
    
# quest_pool
  (50%) Slay the dragon.
    unique
  (50%) Pick up 30 yellow flowers.
  ->quest_giver
```

In this example, the quest to slay the dragon has a 50% chance to appear. Once it shows up, it will never reappear in the story. 

## Comments
A line that starts with `//` is ignored.

## Functions and tags

You can dynamically communicate with your runtime by the way of functions.
To run a `function` you start and finish a line with backticks.

Example:

```cuentitos
# second_day_happy
  You wake up feeling refreshed. Let's see what this day brings.
  `play_sound alarm`
```

The runtime will receive a `function` call with "alarm" as a parameter.

Since this is dynamic, the compiler can't check if the types are the right ones, you'll have to do this yourself.

You can use an arbitrary amount of parameters, they will be passed to the runtime as a vector.

```cuentitos
`play_sound alarm 0.3`
```
It's up to the runtime how to interpret `0.3` in this case, and cuentitos will just pass it through.
You can always use variables as a way to communicate with the runtime and check for types in compile time.

Another way to communicate with your runtime is by using a `tag`. This serves as a marker that you can place on any line.

```cuentitos
# second_day_happy
  * Shelter the dog.
  tag important_decision
```

The runtime will receive a `tag` named `important_decision` so the runtime can, for example, warn the player about the consequences of taking this path.

## Configuration

A user can define variables and their types to use inside conditions.

This is done in the configuration file `cuentitos.toml`.

These can be of type `integer`, `float`, and `bool`, `enum` and `string`.

```toml
[variables]
health = "integer"
money = "integer"
day = { enum = [ "friday", "saturday", "sunday"] }
```

Once defined, you can use them in your story by using the commands `req`, `set` and `freq`.

```
The weekend is finally here!
  req day saturday
```