-- Elevia: initial schema (exploitations, utilisateurs, products, entries, production)

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE user_role AS ENUM ('admin', 'exploitation');

CREATE TABLE exploitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nom TEXT NOT NULL,
    contact TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE utilisateurs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exploitation_id UUID REFERENCES exploitations(id) ON DELETE CASCADE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- an admin belongs to no exploitation, an exploitation user always belongs to exactly one
    CONSTRAINT exploitation_role_consistency CHECK (
        (role = 'admin' AND exploitation_id IS NULL) OR
        (role = 'exploitation' AND exploitation_id IS NOT NULL)
    )
);

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nom TEXT NOT NULL,
    unite TEXT NOT NULL,
    categorie TEXT NOT NULL
);

CREATE TABLE entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exploitation_id UUID NOT NULL REFERENCES exploitations(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    mois DATE NOT NULL,
    quantite NUMERIC(14, 3) NOT NULL CHECK (quantite >= 0),
    cout NUMERIC(14, 2) NOT NULL CHECK (cout >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- resubmitting the same product/month for an exploitation replaces the entry (see upsert in the repository)
    UNIQUE (exploitation_id, product_id, mois)
);

CREATE INDEX idx_entries_exploitation_mois ON entries (exploitation_id, mois);
CREATE INDEX idx_entries_product_mois ON entries (product_id, mois);

CREATE TABLE production (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exploitation_id UUID NOT NULL REFERENCES exploitations(id) ON DELETE CASCADE,
    mois DATE NOT NULL,
    quantite_produite NUMERIC(14, 3) NOT NULL CHECK (quantite_produite >= 0),
    unite TEXT NOT NULL,
    -- optional selling price per unit, used to estimate margin on the dashboard when provided
    prix_unitaire_vente NUMERIC(14, 2) CHECK (prix_unitaire_vente >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (exploitation_id, mois)
);

CREATE INDEX idx_production_exploitation_mois ON production (exploitation_id, mois);
