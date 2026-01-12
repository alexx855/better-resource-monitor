import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { z } from 'astro/zod';

const docs = defineCollection({
  loader: glob({ pattern: "README.md", base: ".." }),
  schema: z.object({})
});

export const collections = { docs };


