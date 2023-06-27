const ROOCH_TEAM: Record<string, AuthorDetails> = {
  jolestar: {
    name: "Jolestar",
    twitterUsername: "jolestar",
    avatar: "/images/people/jolestar.jpg",
  },
  haichao: {
    name: "Haichao",
    twitterUsername: "HaichaoZ",
    avatar: "/images/people/haichao.jpg",
  },
  ren: {
    name: "Ren",
    twitterUsername: "renryderauthor",
    avatar: "/images/people/ren.jpg",
  },
  geometryolife: {
      name: "Joe Chen",
      twitterUsername: "joechendev",
      avatar: "/images/people/joe.jpg"
    }
};

export type Author = keyof typeof ROOCH_TEAM;
export type AuthorDetails = {
  name: string;
  twitterUsername?: string;
  avatar: string;
};

export default ROOCH_TEAM;
