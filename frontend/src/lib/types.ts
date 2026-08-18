export type Role = "admin" | "exploitation";

export interface AuthClaims {
  sub: string;
  email: string;
  role: Role;
  exploitation_id: string | null;
  exp: number;
}

export interface Product {
  id: string;
  nom: string;
  unite: string;
  categorie: string;
}

export interface Entry {
  id: string;
  exploitation_id: string;
  product_id: string;
  mois: string;
  quantite: string;
  cout: string;
}

export interface CreateEntryInput {
  product_id: string;
  mois: string;
  quantite: string;
  cout: string;
}

export interface Production {
  id: string;
  exploitation_id: string;
  mois: string;
  quantite_produite: string;
  unite: string;
  prix_unitaire_vente: string | null;
}

export interface CreateProductionInput {
  mois: string;
  quantite_produite: string;
  unite: string;
  prix_unitaire_vente: string | null;
}

export interface MonthlyStats {
  mois: string;
  total_cost: string;
  quantity_produced: string | null;
  cost_per_unit: string | null;
  estimated_margin: string | null;
}

export interface ExploitationDashboard {
  exploitation_id: string;
  monthly: MonthlyStats[];
}

export interface ProductNeed {
  product_id: string;
  nom: string;
  unite: string;
  total_quantite: string;
}

export interface Quartiles {
  q1: string;
  median: string;
  q3: string;
}

export interface CoopDashboard {
  mois: string;
  intrant_needs: ProductNeed[];
  average_margin: string | null;
  cost_per_unit_quartiles: Quartiles | null;
}

export interface Exploitation {
  id: string;
  nom: string;
  contact: string;
  created_at: string;
}

export interface ExploitationStatus {
  id: string;
  nom: string;
  entries_submitted: boolean;
  production_submitted: boolean;
}

export interface CreateExploitationInput {
  nom: string;
  contact: string;
  email: string;
  password: string;
}

export interface CreateProductInput {
  nom: string;
  unite: string;
  categorie: string;
}

export interface UpdateProductInput {
  nom?: string;
  unite?: string;
  categorie?: string;
}
