use ndarray::{Array2, Array1};
use rand::{Rng, thread_rng};
use std::f64::consts::E;

#[derive(Clone, Debug)]
pub struct NeuralNetwork {
    pub weights: Vec<Array2<f64>>,
    pub biases: Vec<Array1<f64>>,
    pub layer_sizes: Vec<usize>,
    pub fitness: f64,
    pub generation: usize,
}

impl NeuralNetwork {
    pub fn new(layer_sizes: &[usize]) -> Self {
        let mut rng = thread_rng();
        let mut weights = Vec::new();
        let mut biases = Vec::new();
        
        for i in 0..layer_sizes.len() - 1 {
            let w = Array2::from_shape_fn(
                (layer_sizes[i + 1], layer_sizes[i]),
                |_| rng.gen_range(-1.0..1.0) * (2.0 / layer_sizes[i] as f64).sqrt()
            );
            let b = Array1::zeros(layer_sizes[i + 1]);
            weights.push(w);
            biases.push(b);
        }
        
        NeuralNetwork {
            weights,
            biases,
            layer_sizes: layer_sizes.to_vec(),
            fitness: 0.0,
            generation: 0,
        }
    }
    
    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut activation = Array1::from_vec(input.to_vec());
        
        for (w, b) in self.weights.iter().zip(&self.biases) {
            activation = w.dot(&activation) + b;
            activation = activation.mapv(|x| sigmoid(x));
        }
        
        activation.to_vec()
    }
    
    pub fn mutate(&mut self, mutation_rate: f64) {
        let mut rng = thread_rng();
        
        for w in &mut self.weights {
            for elem in w.iter_mut() {
                if rng.gen::<f64>() < mutation_rate {
                    *elem += rng.gen::<f64>() * 0.2 - 0.1; // Simple normal approximation
                }
            }
        }
        
        for b in &mut self.biases {
            for elem in b.iter_mut() {
                if rng.gen::<f64>() < mutation_rate {
                    *elem += rng.gen::<f64>() * 0.2 - 0.1;
                }
            }
        }
    }
    
    pub fn crossover(parent1: &NeuralNetwork, parent2: &NeuralNetwork) -> NeuralNetwork {
        let mut rng = thread_rng();
        let mut child = NeuralNetwork::new(&parent1.layer_sizes);
        
        for (i, (w1, w2)) in parent1.weights.iter().zip(&parent2.weights).enumerate() {
            let shape = w1.shape();
            for idx in 0..shape[0] {
                for jdx in 0..shape[1] {
                    child.weights[i][[idx, jdx]] = if rng.gen_bool(0.5) {
                        w1[[idx, jdx]]
                    } else {
                        w2[[idx, jdx]]
                    };
                }
            }
        }
        
        for (i, (b1, b2)) in parent1.biases.iter().zip(&parent2.biases).enumerate() {
            for (j, (v1, v2)) in b1.iter().zip(b2).enumerate() {
                child.biases[i][j] = if rng.gen_bool(0.5) { *v1 } else { *v2 };
            }
        }
        
        child
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + E.powf(-x))
}

#[derive(Clone, Debug)]
pub struct AttackStrategy {
    pub network: NeuralNetwork,
    pub attack_type: AttackType,
    pub success_rate: f64,
    pub total_attempts: usize,
}

#[derive(Clone, Debug)]
pub enum AttackType {
    PromptInjection,
    ContextOverflow,
    ToolPoisoning,
    DataExfiltration,
    UnicodeSmuggling,
    ChainAttack,
}

impl AttackType {
    pub fn to_one_hot(&self) -> Vec<f64> {
        let mut v = vec![0.0; 6];
        match self {
            AttackType::PromptInjection => v[0] = 1.0,
            AttackType::ContextOverflow => v[1] = 1.0,
            AttackType::ToolPoisoning => v[2] = 1.0,
            AttackType::DataExfiltration => v[3] = 1.0,
            AttackType::UnicodeSmuggling => v[4] = 1.0,
            AttackType::ChainAttack => v[5] = 1.0,
        }
        v
    }
    
    pub fn from_output(output: &[f64]) -> Self {
        let max_idx = output.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        match max_idx {
            0 => AttackType::PromptInjection,
            1 => AttackType::ContextOverflow,
            2 => AttackType::ToolPoisoning,
            3 => AttackType::DataExfiltration,
            4 => AttackType::UnicodeSmuggling,
            _ => AttackType::ChainAttack,
        }
    }
}

pub struct NeuroevolutionEngine {
    pub population: Vec<AttackStrategy>,
    pub generation: usize,
    pub population_size: usize,
    pub mutation_rate: f64,
    pub elite_count: usize,
}

impl NeuroevolutionEngine {
    pub fn new(population_size: usize) -> Self {
        let mut engine = NeuroevolutionEngine {
            population: Vec::new(),
            generation: 0,
            population_size,
            mutation_rate: 0.1,
            elite_count: (population_size as f64 * 0.2) as usize,
        };
        engine.initialize();
        engine
    }
    
    fn initialize(&mut self) {
        let attack_types = vec![
            AttackType::PromptInjection,
            AttackType::ContextOverflow,
            AttackType::ToolPoisoning,
            AttackType::DataExfiltration,
            AttackType::UnicodeSmuggling,
            AttackType::ChainAttack,
        ];
        
        for i in 0..self.population_size {
            let attack_type = attack_types[i % attack_types.len()].clone();
            let layer_sizes = &[10, 20, 15, 6]; // input, hidden1, hidden2, output
            let network = NeuralNetwork::new(layer_sizes);
            
            self.population.push(AttackStrategy {
                network,
                attack_type,
                success_rate: 0.0,
                total_attempts: 0,
            });
        }
    }
    
    pub fn evolve(&mut self) -> Vec<AttackStrategy> {
        self.generation += 1;
        
        // Sort by fitness
        self.population.sort_by(|a, b| {
            b.network.fitness.partial_cmp(&a.network.fitness).unwrap()
        });
        
        let mut new_population = Vec::new();
        
        // Keep elite
        for i in 0..self.elite_count {
            new_population.push(self.population[i].clone());
        }
        
        // Generate offspring through crossover and mutation
        while new_population.len() < self.population_size {
            let parent1 = &self.population[rand::thread_rng().gen_range(0..self.elite_count)];
            let parent2 = &self.population[rand::thread_rng().gen_range(0..self.elite_count)];
            
            let mut child = AttackStrategy {
                network: NeuralNetwork::crossover(&parent1.network, &parent2.network),
                attack_type: if rand::thread_rng().gen_bool(0.5) {
                    parent1.attack_type.clone()
                } else {
                    parent2.attack_type.clone()
                },
                success_rate: 0.0,
                total_attempts: 0,
            };
            
            child.network.mutate(self.mutation_rate);
            child.network.generation = self.generation;
            
            new_population.push(child);
        }
        
        self.population = new_population;
        self.population.clone()
    }
    
    pub fn update_fitness(&mut self, index: usize, success: bool) {
        if index >= self.population.len() {
            return;
        }
        
        let strategy = &mut self.population[index];
        strategy.total_attempts += 1;
        
        if success {
            strategy.success_rate = strategy.success_rate * 
                ((strategy.total_attempts - 1) as f64 / strategy.total_attempts as f64) 
                + 1.0 / strategy.total_attempts as f64;
        } else {
            strategy.success_rate = strategy.success_rate * 
                ((strategy.total_attempts - 1) as f64 / strategy.total_attempts as f64);
        }
        
        // Fitness = success_rate * log(total_attempts + 1)
        strategy.network.fitness = strategy.success_rate * 
            (strategy.total_attempts as f64 + 1.0).ln();
    }
    
    pub fn get_best_strategy(&self) -> Option<&AttackStrategy> {
        self.population.iter().max_by(|a, b| {
            a.network.fitness.partial_cmp(&b.network.fitness).unwrap()
        })
    }
    
    pub fn generate_attack_parameters(&self, strategy: &AttackStrategy, context: &[f64]) -> (AttackType, Vec<u8>) {
        let output = strategy.network.forward(context);
        let attack_type = AttackType::from_output(&output);
        
        // Generate mutation parameters from network output
        let params: Vec<u8> = output.iter()
            .map(|&x| (x * 255.0) as u8)
            .collect();
        
        (attack_type, params)
    }
    
    pub fn get_population_stats(&self) -> (f64, f64, f64) {
        if self.population.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let fitnesses: Vec<f64> = self.population.iter()
            .map(|s| s.network.fitness)
            .collect();
        
        let avg = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        let max = fitnesses.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = fitnesses.iter().cloned().fold(f64::INFINITY, f64::min);
        
        (avg, max, min)
    }
}
