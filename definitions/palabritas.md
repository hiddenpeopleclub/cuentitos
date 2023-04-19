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

A bucket is a set of probable paths that the engine will pick one every time is asked to render that bucket. Let's do a probabilistic bucket with the second option.

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

### Probability of Options

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

#### Options bucket

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

## State changes

## String interpolation

## Configuration
### Variables
#### Bool
#### Integer
#### Float
#### Enum
## Reputations
