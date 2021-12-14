pub fn run()
{
    let article = NewsArticle { title: String::from("Super News"), desc: String::from("bla bla bla bla")};
    let tweet = Tweet { content: String::from("zwitscher zwitscher") };

    println!("{}", article.summarize());
    println!("{}", tweet.summarize());

    println!("{}", article.summarize_lol());
    println!("{}", tweet.summarize_lol());

    println!("{}", article.summarize_whatever());

    let res = do_something_with_summary(&article);
    println!("{}", res);

    let res2 = crazy_trait(&article, &article);
    println!("{}", res2);

    let res3 = give_me_a_trait().summarize();
    println!("{}", res3);

    // TO STRING
    println!("===============");
    println!("{}", article.to_string());
}

pub trait Summary
{
    fn summarize(&self) -> String;
    fn summarize_lol(&self) -> String
    {
        String::from("implement your stuff!, 🤓🤓")
    }

    fn summarize_whatever(&self) -> String
    {
        format!("this is the summary: |{}|", self.summarize())
    }
}



pub struct NewsArticle
{
    pub title: String,
    pub desc: String
}

impl Summary for NewsArticle
{
    fn summarize(&self) -> String
    {
        format!("title: {}, desc: {}", self.title, self.desc)
    }
}

impl std::fmt::Display for NewsArticle
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(self.summarize().as_str())
    }
}

pub struct Tweet
{
    pub content: String
}

impl Summary for Tweet
{
    fn summarize(&self) -> String
    {
        format!("tweet: {}", self.content)
    }
}

fn do_something_with_summary(item: &impl Summary) -> String
{
    format!("do_something_with_summary: |{}|", item.summarize())
}

fn crazy_trait<T: Summary>(item1: &T, item2: &T) -> String
{
    format!("do_something_with_summary: |{}| and again: |{}|", item1.summarize(), item2.summarize())
}

fn give_me_a_trait() -> impl Summary
{
    let tweet = Tweet {content: String::from("bla bla")};
    tweet
}