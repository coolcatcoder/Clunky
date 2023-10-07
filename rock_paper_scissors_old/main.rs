fn main()
{
    let user_choice = RockPaperScissors::Scissors;
    let computer_choice = match user_choice
    {
        RockPaperScissors::Rock => RockPaperScissors::Paper,
        RockPaperScissors::Paper => RockPaperScissors::Scissors,
        RockPaperScissors::Scissors => RockPaperScissors::Rock,
    };

    println!("My {0:?} beats your {1:?}!",computer_choice,user_choice);
}

#[derive(Debug)]
enum RockPaperScissors
{
    Rock,
    Paper,
    Scissors,
}