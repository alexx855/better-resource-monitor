import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { z } from 'astro/zod';

const docs = defineCollection({
  loader: glob({ pattern: "README.md", base: ".." }),
  schema: z.object({})
});

const legal = defineCollection({
  loader: glob({ pattern: "*.md", base: "./src/content/legal" }),
  schema: z.object({
    title: z.string(),
    description: z.string()
  })
});

const contributing = defineCollection({
  loader: glob({ pattern: "CONTRIBUTING.md", base: ".." }),
  schema: z.object({})
});

export const collections = { docs, legal, contributing };


