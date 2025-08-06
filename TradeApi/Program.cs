using TradeApi.Services;
using TradeApi.Services.DataSources;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddControllers();
builder.Services.AddOpenApi();
builder.Services.AddSingleton<TradeDataService>();
builder.Services.AddSingleton<QueryProcessor>();

// Add HttpClient for external APIs
builder.Services.AddHttpClient();

// Register data sources
builder.Services.AddSingleton<IDataSource, SqlServerDataSource>();
builder.Services.AddSingleton<IDataSource, PublicApiDataSource>();
builder.Services.AddSingleton<IDataSource, FileDataSource>();

// Register the router
builder.Services.AddSingleton<DataSourceRouter>();

// Add session for simple caching (in production, use Redis or similar)
builder.Services.AddDistributedMemoryCache();
builder.Services.AddSession(options =>
{
    options.IdleTimeout = TimeSpan.FromMinutes(30);
    options.Cookie.HttpOnly = true;
    options.Cookie.IsEssential = true;
});

// Configure CORS
builder.Services.AddCors(options =>
{
    options.AddPolicy("AllowAll",
        builder =>
        {
            builder.AllowAnyOrigin()
                   .AllowAnyMethod()
                   .AllowAnyHeader();
        });
});

var app = builder.Build();

// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
}

app.UseCors("AllowAll");
app.UseHttpsRedirection();
app.UseSession(); // Enable session for caching
app.MapControllers();

app.Run();

// Make the implicit Program class accessible to integration tests
public partial class Program { }
