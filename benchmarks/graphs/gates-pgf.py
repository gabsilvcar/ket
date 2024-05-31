import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

# Ensure Matplotlib uses PGF backend for LaTeX output
plt.rcParams.update({
    "pgf.texsystem": "pdflatex",
    "font.family": "serif",
    "text.usetex": True,
    "pgf.rcfonts": False
})

# Load the CSV file using Pandas
df = pd.read_csv('benchmarks-random.csv')
palette = 'magma'

# Define a dictionary to map English names to Brazilian Portuguese
name_mapping = {
    'original-gates': 'original',
    'basic-gates': 'original com portas básicas',
    'quizx-clifford-simplified-gates': 'simplificação de clifford quizx',
    'pyzx-clifford-simplified-gates': 'simplificação de clifford pyzx',
    'pyzx-full-reduction-gates': 'redução completa pyzx'
}

# Reshape the DataFrame to long format and apply the name mapping
df_long = df.melt(id_vars='p(t)', value_vars=list(name_mapping.keys()), var_name='Method', value_name='Number of Gates')
df_long['Method'] = df_long['Method'].map(name_mapping)

# Create a scatter plot with different colors and labels for each method
sns.lineplot(x='p(t)', y='Number of Gates', hue='Method', data=df_long, palette=palette)
sns.scatterplot(x='p(t)', y='Number of Gates', hue='Method', data=df_long, palette=palette, marker='s', legend=False)

# Customize legend
plt.legend(title='Método de Redução', frameon=True, fontsize='small')

plt.xlabel('Proporção de portas T')
plt.ylabel('Quantidade de portas quânticas')

# Save the plot as a PGF file
plt.savefig('quantum_gates_reduction.pgf')