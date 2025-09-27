var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddControllers()
    .AddNewtonsoftJson(); // Use Newtonsoft for better JToken support

// Add CORS for testing from browser/SQL CLI
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

// Configure JSON options
builder.Services.ConfigureHttpJsonOptions(options =>
{
    options.SerializerOptions.WriteIndented = true;
});

var app = builder.Build();

// Configure the HTTP request pipeline.
app.UseCors("AllowAll");

app.UseRouting();

app.MapControllers();

// Add welcome page with API info
app.MapGet("/", () => new
{
    service = "JSON Selector Service",
    version = "1.0",
    description = "Transform hierarchical JSON into tabular data",
    endpoints = new[]
    {
        new { path = "/api/query/upload", method = "POST", description = "Simple query with file upload" },
        new { path = "/api/query/direct", method = "POST", description = "Query with JSON in request body" },
        new { path = "/api/messagequery/upload", method = "POST", description = "Message type-aware query" },
        new { path = "/api/messagequery/example/fix", method = "POST", description = "FIX message example query" }
    },
    selectorExamples = new[]
    {
        "body.Symbol - Navigate to nested field",
        "Parties[PartyRole=11].PartyID - Filter array and get field",
        "AllocGrp:sum(AllocQty) - Aggregate array values",
        "Parties[0].PartyID - Get first element",
        "AllocGrp[*].AllocAccount - Get all values"
    }
});

var port = Environment.GetEnvironmentVariable("PORT") ?? "5050";
Console.WriteLine($"JSON Selector Service starting on http://localhost:{port}");
app.Run($"http://localhost:{port}");